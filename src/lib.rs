use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tokio;
use tokio::sync::mpsc::{self, Sender};
use futures::StreamExt;
use std::env;
use toml::value::Table;

pub fn get_api_key() -> Result<String, Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let secrets_path: PathBuf = current_dir.join("secrets.toml");
    println!("secrets_path: {:#?}", secrets_path);

    let toml_content = fs::read_to_string(secrets_path).expect("Failed to read the .toml file");
    let keychain: Table = toml::from_str(&toml_content)?;
    let openai_keys = keychain.get("openai").ok_or("Missing 'openai' section in secrets.toml")?;
    let api_key = openai_keys.get("api_key").ok_or("Missing 'api_key' in 'openai' section")?.as_str().ok_or("api_key is not a string")?;

    Ok(api_key.to_string())
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
    tx: Sender<String>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
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
    if let Some(stream) = params.stream {
        body["stream"] = serde_json::json!(stream);
    }

    let client = Client::new();

    let mut response = client.post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await?;

    if params.stream.unwrap_or(false) {
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
                            eprintln!("Failed to parse JSON: {}", e);
                        }
                    }
                }
            }
        }
        Ok(serde_json::json!({"status": "streaming complete"}))
    } else {
        let response_text = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text)?;
        Ok(response_json)
    }
}
