use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tokio;
use std::env;
use toml::value::Table;

pub fn get_api_key() -> Result<String, Box<dyn std::error::Error>> {
    // let username = env::var("USERNAME").unwrap_or_else(|_| String::from("default"));
    // let secrets_path = format!(r"C:\\Users\\{}\\secrets.toml", username);
    let current_dir = env::current_dir()?;
    let secrets_path: PathBuf = current_dir.join("secrets.toml");
    println!("secrets_path: {:#?}",secrets_path);

    let toml_content = fs::read_to_string(secrets_path).expect("Failed to read the .toml file");
    // Parse the TOML content into a Table
    let keychain: Table = toml::from_str(&toml_content)?;
    // Access the "openai" section (if it exists)
    let openai_keys = keychain.get("openai").ok_or("Missing 'openai' section in secrets.toml")?;
    let api_key = openai_keys.get("api_key").ok_or("Missing 'api_key' in 'openai' section")?.as_str().ok_or("api_key is not a string")?;
    
    // Convert &str to String
    Ok(api_key.to_string())
}

pub async fn chat_completion(messages: Vec<Value>, model: String) -> Result<Value, Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let body = serde_json::json!({
        "model": model,
        "messages": messages
    });
    // Create an async HTTP client
    let client = Client::new();

    // Send the request
    let response = client.post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await?;
    let response_text = response.text().await?;
    let response_json: Value = serde_json::from_str(&response_text)?;
    Ok(response_json)
}

