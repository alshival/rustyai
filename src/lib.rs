use reqwest::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tokio;
use tokio::sync::mpsc::{self, Sender};
use futures::StreamExt;
use std::env;
use toml::value::Table;

pub fn get_api_key() -> Result<(String, String, String), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let secrets_path: PathBuf = current_dir.join("secrets.toml");

    let toml_content = fs::read_to_string(secrets_path).expect("Failed to read the .toml file");
    let keychain: Table = toml::from_str(&toml_content)?;
    let openai_keys = keychain.get("openai").ok_or("Missing 'openai' section in secrets.toml")?;
    let api_key = openai_keys.get("api_key").ok_or("Missing 'api_key' in 'openai' section")?.as_str().ok_or("api_key is not a string")?;
    let organization = openai_keys.get("organization").ok_or("Missing 'organization' in 'openai' section")?.as_str().ok_or("organization is not a string")?;
    let project = openai_keys.get("project").ok_or("Missing 'project' in 'openai' section")?.as_str().ok_or("project is not a string")?;

    Ok((api_key.to_string(), organization.to_string(), project.to_string()))
}

pub fn create_client() -> Result<Client, Box<dyn std::error::Error>> {
    let (api_key, organization, project) = get_api_key()?;

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", api_key))?);
    headers.insert("OpenAI-Organization", HeaderValue::from_str(&organization)?);
    headers.insert("OpenAI-Project", HeaderValue::from_str(&project)?);

    let client = Client::builder()
        .default_headers(headers)
        .build()?;

    Ok(client)
}

#[derive(Default)]
pub struct ChatCompletionParams {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
    pub stream: Option<bool>,
}

pub async fn chat_completion(
    messages: Vec<Value>,
    model: String,
    params: ChatCompletionParams,
) -> Result<Value, Box<dyn std::error::Error>> {
    let client = create_client()?;
    let mut body = serde_json::json!({
        "model": model,
        "messages": messages
    });

    if let Some(max_tokens) = params.max_tokens {
        body["max_tokens"] = serde_json::json!(max_tokens);
    }
    if let Some(temperature) = params.temperature {
        body["temperature"] = serde_json::json!(temperature);
    }
    if let Some(top_p) = params.top_p {
        body["top_p"] = serde_json::json!(top_p);
    }
    if let Some(frequency_penalty) = params.frequency_penalty {
        body["frequency_penalty"] = serde_json::json!(frequency_penalty);
    }
    if let Some(presence_penalty) = params.presence_penalty {
        body["presence_penalty"] = serde_json::json!(presence_penalty);
    }

    let response = client.post("https://api.openai.com/v1/chat/completions")
        .json(&body)
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_json: Value = serde_json::from_str(&response_text)?;
    Ok(response_json)
}

pub async fn chat_completion_stream(
    messages: Vec<Value>,
    model: String,
    params: ChatCompletionParams,
    tx: Sender<String>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let client = create_client()?;
    let mut body = serde_json::json!({
        "model": model,
        "messages": messages
    });

    if let Some(max_tokens) = params.max_tokens {
        body["max_tokens"] = serde_json::json!(max_tokens);
    }
    if let Some(temperature) = params.temperature {
        body["temperature"] = serde_json::json!(temperature);
    }
    if let Some(top_p) = params.top_p {
        body["top_p"] = serde_json::json!(top_p);
    }
    if let Some(frequency_penalty) = params.frequency_penalty {
        body["frequency_penalty"] = serde_json::json!(frequency_penalty);
    }
    if let Some(presence_penalty) = params.presence_penalty {
        body["presence_penalty"] = serde_json::json!(presence_penalty);
    }
    body["stream"] = serde_json::json!(true);

    let mut response = client.post("https://api.openai.com/v1/chat/completions")
        .json(&body)
        .send()
        .await?;

    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        let chunk_str = String::from_utf8_lossy(&chunk).to_string();
        
        // Process the streamed data
        for line in chunk_str.split("data: ").filter(|line| !line.trim().is_empty()) {
            if line.trim() == "[DONE]" {
                tx.send("[DONE]".to_string()).await?;
            } else {
                match serde_json::from_str::<Value>(line) {
                    Ok(json_value) => {
                        if let Some(content) = json_value["choices"][0]["delta"]["content"].as_str() {
                            tx.send(content.to_string()).await?;
                        }
                    },
                    Err(e) => {
                        //eprintln!("Failed to parse JSON: {} -> {}", e, line);
                    }
                }
            }
        }
    }
    Ok(serde_json::json!({"status": "streaming complete"}))
}
