use reqwest::Client;
use serde::Deserialize;
use std::fs;
use tokio;
use std::env;
use toml::value::Table;
use rustyai;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let messages = vec![
        serde_json::json!({
            "role": "user",
            "content": r#"
                Remember this phrase: In a field of horses, be a unicorn.
                I will ask you to repeat it.
            "#
        }),
        serde_json::json!({
            "role": "user",
            "content": "What did I ask you to remember?"
        })
    ];
    // Call chat_completion_stream (streaming)
    let params_stream = rustyai::ChatCompletionParams {
        max_tokens: Some(100),
        temperature: Some(0.7),
        top_p: Some(0.9),
        frequency_penalty: Some(0.0),
        presence_penalty: Some(0.0),
        ..Default::default()
    };

    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move {
        let _ = rustyai::chat_completion_stream(messages, "gpt-3.5-turbo".to_string(), params_stream, tx).await;
    });

    while let Some(content) = rx.recv().await {
        if content == "[DONE]" {
            break;
        }
        println!("{}", content);
    }

    Ok(())
}
