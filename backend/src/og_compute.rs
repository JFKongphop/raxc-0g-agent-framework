/*!
0G Compute Client — LLM inference interface for 0G Compute network.

This replaces OpenAI API with 0G Compute for reasoning and report generation.
Uses OpenAI-compatible chat completions API.
*/

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// 0G Compute client for LLM inference
#[derive(Clone)]
pub struct OgComputeClient {
  endpoint: String,
  model: String,
  api_key: Option<String>,
  http: Client,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
  role: String,
  content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
  model: String,
  messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct ChatChoice {
  message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
  choices: Vec<ChatChoice>,
}

impl OgComputeClient {
  /// Create a new 0G Compute client
  pub fn new(endpoint: String, model: String) -> Self {
    Self {
      endpoint,
      model,
      api_key: None,
      http: Client::new(),
    }
  }

  /// Create a new 0G Compute client with API key
  pub fn with_api_key(endpoint: String, model: String, api_key: String) -> Self {
    Self {
      endpoint,
      model,
      api_key: Some(api_key),
      http: Client::new(),
    }
  }

  /// Run inference on the 0G Compute network
  /// Uses OpenAI-compatible chat completions API format
  pub async fn infer(&self, prompt: &str) -> Result<String> {
    let req = ChatCompletionRequest {
      model: self.model.clone(),
      messages: vec![
        ChatMessage {
          role: "system".to_string(),
          content: "You are a smart contract security expert.".to_string(),
        },
        ChatMessage {
          role: "user".to_string(),
          content: prompt.to_string(),
        },
      ],
    };

    let mut request = self.http.post(&self.endpoint).json(&req);

    // Add Bearer token if API key is provided
    if let Some(api_key) = &self.api_key {
      request = request.bearer_auth(api_key);
    }

    let resp: ChatCompletionResponse = request
      .send()
      .await
      .context("Failed to send inference request to 0G Compute")?
      .error_for_status()
      .context("0G Compute inference request returned error")?
      .json()
      .await
      .context("Failed to parse 0G Compute inference response")?;

    Ok(
      resp
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .unwrap_or_default(),
    )
  }
}
