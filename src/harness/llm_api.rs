use crate::harness::context_window::ProviderContextLimits;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct OpenAiRestConfig {
    pub base_url: String,
    pub provider_name: String,
    pub model_name: String,
    pub max_context_tokens: usize,
    pub reserved_output_tokens: usize,
}

impl OpenAiRestConfig {
    pub fn llama_cpp(base_url: impl Into<String>, model_name: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            provider_name: "llama.cpp".to_string(),
            model_name: model_name.into(),
            max_context_tokens: 8192,
            reserved_output_tokens: 1024,
        }
    }

    pub fn context_limits(&self) -> ProviderContextLimits {
        ProviderContextLimits {
            provider_name: self.provider_name.clone(),
            model_name: self.model_name.clone(),
            max_context_tokens: self.max_context_tokens,
            reserved_output_tokens: self.reserved_output_tokens,
        }
    }
}

pub struct LlmApiClient {
    http: Client,
    config: OpenAiRestConfig,
}

impl LlmApiClient {
    pub fn new(config: OpenAiRestConfig) -> Self {
        Self {
            http: Client::new(),
            config,
        }
    }

    pub fn context_limits(&self) -> ProviderContextLimits {
        self.config.context_limits()
    }

    pub async fn complete_chat(
        &self,
        messages: Vec<ChatMessage>,
        max_tokens: usize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let request = ChatCompletionRequest {
            model: self.config.model_name.clone(),
            messages,
            max_tokens,
            stream: false,
        };

        let response = self
            .http
            .post(format!("{}/v1/chat/completions", self.config.base_url))
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json::<ChatCompletionResponse>()
            .await?;

        let content = response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .unwrap_or_default();

        Ok(content)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: usize,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    message: ChatMessage,
}
