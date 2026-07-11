use crate::harness::context_window::ProviderContextLimits;
use reqwest::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::RwLock;
use std::time::Duration;
#[cfg(test)]
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use tokio::time::sleep;

const DEFAULT_HTTP_TIMEOUT_SECS: u64 = 90;
const TRANSIENT_RETRY_DELAY_SECS: u64 = 5;
const MAX_TRANSIENT_ATTEMPTS: usize = 2;

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

    pub fn apply_capabilities(&mut self, capabilities: &LlmBackendCapabilities) {
        if let Some(provider_name) = capabilities.provider_name.as_deref() {
            if !provider_name.trim().is_empty() {
                self.provider_name = provider_name.trim().to_string();
            }
        }

        if let Some(model_name) = capabilities.model_id.as_deref() {
            if !model_name.trim().is_empty() {
                self.model_name = model_name.trim().to_string();
            }
        }

        if let Some(max_context_tokens) = capabilities.max_context_tokens {
            if max_context_tokens > 0 {
                self.max_context_tokens = max_context_tokens;
            }
        }

        if let Some(default_max_tokens) = capabilities.default_max_tokens {
            if default_max_tokens > 0 {
                self.reserved_output_tokens = default_max_tokens;
            }
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
    config: RwLock<OpenAiRestConfig>,
    #[cfg(test)]
    scripted_responses: Option<Arc<Mutex<VecDeque<String>>>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LlmBackendCapabilities {
    #[serde(default)]
    pub provider_name: Option<String>,
    #[serde(default)]
    pub model_id: Option<String>,
    #[serde(default)]
    pub max_context_tokens: Option<usize>,
    #[serde(default)]
    pub default_max_tokens: Option<usize>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(tag = "type")]
pub enum ChatCompletionResponseFormat {
    #[serde(rename = "json_object")]
    JsonObject,
}

impl LlmApiClient {
    pub fn new(config: OpenAiRestConfig) -> Self {
        Self {
            http: Client::builder()
                .timeout(Duration::from_secs(DEFAULT_HTTP_TIMEOUT_SECS))
                .build()
                .expect("reqwest client should build"),
            config: RwLock::new(config),
            #[cfg(test)]
            scripted_responses: None,
        }
    }

    #[cfg(test)]
    pub fn scripted_for_tests(config: OpenAiRestConfig, responses: Vec<String>) -> Self {
        Self {
            http: Client::builder()
                .timeout(Duration::from_secs(DEFAULT_HTTP_TIMEOUT_SECS))
                .build()
                .expect("reqwest client should build"),
            config: RwLock::new(config),
            scripted_responses: Some(Arc::new(Mutex::new(
                responses.into_iter().collect::<VecDeque<_>>(),
            ))),
        }
    }

    pub fn context_limits(&self) -> ProviderContextLimits {
        self.config_snapshot().context_limits()
    }

    pub fn current_model_name(&self) -> String {
        self.config_snapshot().model_name
    }

    pub fn set_model_name(&self, model_name: impl Into<String>) {
        let mut config = self.config.write().expect("llm config lock poisoned");
        config.model_name = model_name.into();
    }

    pub async fn fetch_capabilities(
        base_url: &str,
    ) -> Result<LlmBackendCapabilities, Box<dyn std::error::Error>> {
        let http = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_HTTP_TIMEOUT_SECS))
            .build()
            .expect("reqwest client should build");
        let response = http
            .get(format!("{base_url}/v1/capabilities"))
            .send()
            .await
            .map_err(|err| {
                LlmApiError::message(format!(
                    "request to {base_url}/v1/capabilities failed: {err}"
                ))
            })?;

        let status = response.status();
        let response_text = response.text().await.map_err(|err| {
            LlmApiError::message(format!(
                "failed reading response body from {base_url}/v1/capabilities: {err}"
            ))
        })?;

        if !status.is_success() {
            return Err(LlmApiError::HttpStatus {
                status,
                body_snippet: truncate_body(&response_text, 800),
            }
            .into());
        }

        let payload =
            serde_json::from_str::<CapabilitiesResponse>(&response_text).map_err(|err| {
                LlmApiError::ResponseParse {
                    message: format!("failed to parse capabilities response: {err}"),
                    body_snippet: truncate_body(&response_text, 800),
                }
            })?;

        payload.capabilities.ok_or_else(|| {
            LlmApiError::message("capabilities response did not include a `capabilities` object")
                .into()
        })
    }

    pub async fn fetch_available_models(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let config = self.config_snapshot();
        let response = self
            .http
            .get(format!("{}/v1/models", config.base_url))
            .send()
            .await
            .map_err(|err| {
                LlmApiError::message(format!(
                    "request to {}/v1/models failed: {err}",
                    config.base_url
                ))
            })?;

        let status = response.status();
        let response_text = response.text().await.map_err(|err| {
            LlmApiError::message(format!(
                "failed reading response body from {}/v1/models: {err}",
                config.base_url
            ))
        })?;

        if !status.is_success() {
            return Err(LlmApiError::HttpStatus {
                status,
                body_snippet: truncate_body(&response_text, 800),
            }
            .into());
        }

        let payload = serde_json::from_str::<ModelsResponse>(&response_text).map_err(|err| {
            LlmApiError::ResponseParse {
                message: format!("failed to parse models response: {err}"),
                body_snippet: truncate_body(&response_text, 800),
            }
        })?;

        let mut models = payload
            .data
            .into_iter()
            .map(|model| model.id.trim().to_string())
            .filter(|id| !id.is_empty())
            .collect::<Vec<_>>();
        models.sort();
        models.dedup();
        Ok(models)
    }

    pub async fn complete_chat(
        &self,
        messages: Vec<ChatMessage>,
        max_tokens: usize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.complete_chat_with_response_format(messages, max_tokens, None)
            .await
    }

    pub async fn complete_chat_with_response_format(
        &self,
        messages: Vec<ChatMessage>,
        max_tokens: usize,
        response_format: Option<ChatCompletionResponseFormat>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        #[cfg(test)]
        if let Some(scripted) = self.scripted_responses.as_ref() {
            let mut queued = scripted.lock().expect("lock scripted llm responses");
            let response = queued.pop_front().ok_or_else(|| {
                LlmApiError::message(format!(
                    "scripted llm responses exhausted for request with {} messages and max_tokens {}",
                    messages.len(),
                    max_tokens
                ))
            })?;
            return Ok(response);
        }

        let config = self.config_snapshot();
        let request = ChatCompletionRequest {
            base_url: config.base_url,
            model: config.model_name,
            messages,
            max_tokens,
            stream: false,
            response_format,
        };

        let mut last_error: Option<LlmApiError> = None;

        for attempt in 1..=MAX_TRANSIENT_ATTEMPTS {
            match self.try_complete_chat(&request).await {
                Ok(content) => return Ok(content),
                Err(err) => {
                    let should_retry = err.is_retryable() && attempt < MAX_TRANSIENT_ATTEMPTS;
                    last_error = Some(err);
                    if should_retry {
                        sleep(Duration::from_secs(TRANSIENT_RETRY_DELAY_SECS)).await;
                        continue;
                    }
                    break;
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| {
                LlmApiError::message("llm request failed without a captured transport error")
            })
            .into())
    }

    async fn try_complete_chat(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<String, LlmApiError> {
        let response = self
            .http
            .post(format!("{}/v1/chat/completions", request.base_url))
            .json(request)
            .send()
            .await
            .map_err(|err| LlmApiError::Transport {
                message: format!(
                    "request to {}/v1/chat/completions failed: {err}",
                    request.base_url
                ),
            })?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|err| LlmApiError::Transport {
                message: format!(
                    "failed reading response body from {}/v1/chat/completions: {err}",
                    request.base_url
                ),
            })?;

        if !status.is_success() {
            return Err(LlmApiError::HttpStatus {
                status,
                body_snippet: truncate_body(&response_text, 800),
            });
        }

        let response =
            serde_json::from_str::<ChatCompletionResponse>(&response_text).map_err(|err| {
                LlmApiError::ResponseParse {
                    message: format!("failed to parse chat completion response: {err}"),
                    body_snippet: truncate_body(&response_text, 800),
                }
            })?;

        Ok(response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .unwrap_or_default())
    }

    fn config_snapshot(&self) -> OpenAiRestConfig {
        self.config
            .read()
            .expect("llm config lock poisoned")
            .clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    #[serde(skip_serializing)]
    base_url: String,
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: usize,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ChatCompletionResponseFormat>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct CapabilitiesResponse {
    capabilities: Option<LlmBackendCapabilities>,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    #[serde(default)]
    data: Vec<ModelDescriptor>,
}

#[derive(Debug, Deserialize)]
struct ModelDescriptor {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    message: ChatMessage,
}

#[derive(Debug)]
enum LlmApiError {
    Transport {
        message: String,
    },
    HttpStatus {
        status: StatusCode,
        body_snippet: String,
    },
    ResponseParse {
        message: String,
        body_snippet: String,
    },
    Message(String),
}

impl LlmApiError {
    fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Transport { .. }
                | Self::HttpStatus {
                    status: StatusCode::TOO_MANY_REQUESTS
                        | StatusCode::INTERNAL_SERVER_ERROR
                        | StatusCode::BAD_GATEWAY
                        | StatusCode::SERVICE_UNAVAILABLE
                        | StatusCode::GATEWAY_TIMEOUT,
                    ..
                }
        )
    }
}

impl fmt::Display for LlmApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport { message } => write!(f, "{message}"),
            Self::HttpStatus {
                status,
                body_snippet,
            } => {
                if body_snippet.is_empty() {
                    write!(f, "HTTP {} from local LLM backend", status)
                } else {
                    write!(
                        f,
                        "HTTP {} from local LLM backend. Response body:\n{}",
                        status, body_snippet
                    )
                }
            }
            Self::ResponseParse {
                message,
                body_snippet,
            } => {
                if body_snippet.is_empty() {
                    write!(f, "{message}")
                } else {
                    write!(f, "{message}\nResponse body:\n{body_snippet}")
                }
            }
            Self::Message(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for LlmApiError {}

fn truncate_body(body: &str, max_chars: usize) -> String {
    let trimmed = body.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let mut out = trimmed
        .chars()
        .take(max_chars.saturating_sub(3))
        .collect::<String>();
    out.push_str("...");
    out
}

#[cfg(test)]
mod tests {
    use super::{CapabilitiesResponse, LlmApiError, ModelsResponse, truncate_body};
    use reqwest::StatusCode;

    #[test]
    fn truncates_http_body_for_diagnostics() {
        assert_eq!(truncate_body("abcdefghij", 6), "abc...");
    }

    #[test]
    fn retries_transient_http_errors_only() {
        let retryable = LlmApiError::HttpStatus {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body_snippet: "busy".to_string(),
        };
        let non_retryable = LlmApiError::HttpStatus {
            status: StatusCode::BAD_REQUEST,
            body_snippet: "bad request".to_string(),
        };

        assert!(retryable.is_retryable());
        assert!(!non_retryable.is_retryable());
    }

    #[test]
    fn parses_capabilities_response() {
        let response = serde_json::from_str::<CapabilitiesResponse>(
            r#"{
                "ok": true,
                "provider_name": "llama.cpp",
                "default_model": "gemma-4-local",
                "capabilities": {
                    "provider_name": "llama.cpp",
                    "model_id": "gemma-4-local",
                    "max_context_tokens": 32768,
                    "default_max_tokens": 1024
                }
            }"#,
        )
        .expect("capabilities response should parse");

        let capabilities = response.capabilities.expect("capabilities should exist");
        assert_eq!(capabilities.max_context_tokens, Some(32768));
        assert_eq!(capabilities.default_max_tokens, Some(1024));
    }

    #[test]
    fn parses_models_response() {
        let response = serde_json::from_str::<ModelsResponse>(
            r#"{
                "data": [
                    { "id": "gemma-4-local" },
                    { "id": "qwen-3.5-local" }
                ]
            }"#,
        )
        .expect("models response should parse");

        let ids = response
            .data
            .into_iter()
            .map(|model| model.id)
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["gemma-4-local", "qwen-3.5-local"]);
    }
}
