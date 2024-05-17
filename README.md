# RustyAI 
A Rust OpenAI API Client by [Alshival's Data Service] for interacting with OpenAI's API, allowing for chat completions and streaming chat completions. 

## Features
- Retrieve OpenAI API key from a `secrets.toml` file.
- Perform chat completions with customizable parameters.
- Stream chat completions and process streamed data.

## Usage

### `secrets.toml` File

Create a `secrets.toml` file in the active directory of your project with the following structure:

```toml
[openai]
api_key = "your_openai_api_key_here"
```

### Example Code

Here's an example of how to use the library:

```rust
use reqwest::Client;
use serde_json::Value;
use tokio::sync::mpsc;
use rustyai::{ChatCompletionParams, chat_completion, chat_completion_stream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Chat Completion Example
    let messages = vec![
        serde_json::json!({"role": "system", "content": "You are a helpful assistant."}),
        serde_json::json!({"role": "user", "content": "What is the weather today?"})
    ];

    let params = ChatCompletionParams {
        max_tokens: Some(50),
        temperature: Some(0.7),
        ..Default::default()
    };

    let response = chat_completion(messages.clone(), "gpt-3.5-turbo".to_string(), params).await?;
    println!("Chat Completion Response: {}", response);

    // Chat Completion Stream Example
    let (tx, mut rx) = mpsc::channel(100);
    tokio::spawn(async move {
        while let Some(content) = rx.recv().await {
            println!("Streamed Content: {}", content);
        }
    });

    let stream_params = ChatCompletionParams {
        stream: Some(true),
        ..Default::default()
    };

    let _ = chat_completion_stream(messages, "gpt-3.5-turbo".to_string(), stream_params, tx).await?;

    Ok(())
}
```

## Library Functions

### `get_api_key() -> Result<String, Box<dyn std::error::Error>>`

Retrieves the OpenAI API key from the `secrets.toml` file.

### `chat_completion(messages: Vec<Value>, model: String, params: ChatCompletionParams) -> Result<Value, Box<dyn std::error::Error>>`

Performs a chat completion using the OpenAI API.

- `messages`: A vector of JSON values representing the messages.
- `model`: The model to use for the completion.
- `params`: A struct containing optional parameters for the completion.

### `chat_completion_stream(messages: Vec<Value>, model: String, params: ChatCompletionParams, tx: Sender<String>) -> Result<Value, Box<dyn std::error::Error>>`

Streams chat completions and sends the content through a channel.

- `messages`: A vector of JSON values representing the messages.
- `model`: The model to use for the completion.
- `params`: A struct containing optional parameters for the completion.
- `tx`: A channel sender to transmit streamed content.

## ChatCompletionParams Struct

A struct to specify optional parameters for chat completions:

- `max_tokens: Option<u32>`
- `temperature: Option<f64>`
- `top_p: Option<f64>`
- `frequency_penalty: Option<f64>`
- `presence_penalty: Option<f64>`
- `stream: Option<bool>`

## More to come!
We plan on incorporating more features in the near future, especially those around openAI's newer omni models. Vision and Audio support are in the works.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.
