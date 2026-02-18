use crate::models::router::Backend;
use crate::models::settings::AiSettings;
use log::{debug, error, info};
use reqwest::Client as HttpClient;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::{anthropic, cohere, groq, mistral, openai, perplexity, xai};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiServiceError {
    #[error("API key not found in config or for env var {0}")]
    ApiKeyNotFound(String),

    #[error("Unsupported AI provider: {0}")]
    UnsupportedProvider(String),

    #[error(transparent)]
    RigError(#[from] rig::completion::PromptError),

    #[error("Error from AI provider: {0}")]
    ProviderError(String),
}

#[derive(Debug, Error)]
pub enum PredictBackendError {
    #[error("AI execution failed")]
    AiError(#[from] AiServiceError),

    #[error("Failed to parse AI response: {0}")]
    ResponseParseError(String),

    #[error("AI returned an invalid or out-of-bounds backend index from response: {0}")]
    InvalidIndex(String),
}

/// Service for handling AI-related operations.
pub struct AiService {
    pub settings: AiSettings,
}

impl AiService {
    /// Creates a new instance of AiService.
    pub fn new(settings: AiSettings) -> Self {
        Self { settings }
    }

    /// Performs a simple completion request to test the integration.
    pub async fn ask(&self, prompt: &str) -> Result<String, AiServiceError> {
        let provider = self.settings.provider.to_lowercase();
        let model = &self.settings.model;

        // Common preamble for the agent
        let preamble = "You are a helpful AI assistant integrated into the Kairos-rs gateway.";

        // Helper to get API key from config or env
        let get_key = |env_var: &str| -> Result<String, AiServiceError> {
            self.settings
                .api_key
                .clone()
                .or_else(|| std::env::var(env_var).ok())
                .ok_or_else(|| AiServiceError::ApiKeyNotFound(env_var.to_string()))
        };

        let response = {
            macro_rules! delegate_prompt {
                ($client:ty, $key_env:literal, $provider_name:literal) => {{
                    let key = get_key($key_env)?;
                    let client = <$client>::new(&key)
                        .map_err(|e| AiServiceError::ProviderError(e.to_string()))?;
                    let agent = client.agent(model).preamble(preamble).build();
                    info!("Sending prompt to {} model: {}", $provider_name, model);
                    agent
                        .prompt(prompt)
                        .await
                        .map_err(AiServiceError::RigError)?
                }};
            }

            match provider.as_str() {
                "openai" => {
                    delegate_prompt!(openai::Client<HttpClient>, "OPENAI_API_KEY", "OpenAI")
                }
                "anthropic" => {
                    delegate_prompt!(
                        anthropic::Client<HttpClient>,
                        "ANTHROPIC_API_KEY",
                        "Anthropic"
                    )
                }
                "cohere" => {
                    delegate_prompt!(cohere::Client<HttpClient>, "COHERE_API_KEY", "Cohere")
                }
                "perplexity" => {
                    delegate_prompt!(
                        perplexity::Client<HttpClient>,
                        "PERPLEXITY_API_KEY",
                        "Perplexity"
                    )
                }
                "mistral" => {
                    delegate_prompt!(mistral::Client<HttpClient>, "MISTRAL_API_KEY", "Mistral")
                }
                "groq" => delegate_prompt!(groq::Client<HttpClient>, "GROQ_API_KEY", "Groq"),
                "xai" => delegate_prompt!(xai::Client<HttpClient>, "XAI_API_KEY", "xAI"),
                _ => {
                    let msg = format!("Unsupported AI provider: {}", provider);
                    error!("{}", msg);
                    return Err(AiServiceError::UnsupportedProvider(provider));
                }
            }
        };

        Ok(response)
    }

    /// Internal helper to execute a prompt against the configured provider
    async fn execute_prompt(
        &self,
        prompt: &str,
        preamble: &str,
        provider_override: Option<&str>,
        model_override: Option<&str>,
    ) -> Result<String, AiServiceError> {
        let provider = provider_override
            .unwrap_or(&self.settings.provider)
            .to_lowercase();
        let model = model_override.unwrap_or(&self.settings.model);

        // Helper to get API key from config or env
        let get_key = |env_var: &str| -> Result<String, AiServiceError> {
            self.settings
                .api_key
                .clone()
                .or_else(|| std::env::var(env_var).ok())
                .ok_or_else(|| AiServiceError::ApiKeyNotFound(env_var.to_string()))
        };

        macro_rules! delegate_prompt {
            ($client:ty, $key_env:literal, $provider_name:literal, $preamble:expr) => {{
                let key = get_key($key_env)?;
                let client = <$client>::new(&key)
                    .map_err(|e| AiServiceError::ProviderError(e.to_string()))?;
                let agent = client.agent(model).preamble($preamble).build();
                debug!("Sending prompt to {} model: {}", $provider_name, model);
                agent
                    .prompt(prompt)
                    .await
                    .map_err(AiServiceError::RigError)?
            }};
        }

        let response = match provider.as_str() {
            "openai" => {
                delegate_prompt!(
                    openai::Client<HttpClient>,
                    "OPENAI_API_KEY",
                    "OpenAI",
                    preamble
                )
            }
            "anthropic" => {
                delegate_prompt!(
                    anthropic::Client<HttpClient>,
                    "ANTHROPIC_API_KEY",
                    "Anthropic",
                    preamble
                )
            }
            "cohere" => {
                delegate_prompt!(
                    cohere::Client<HttpClient>,
                    "COHERE_API_KEY",
                    "Cohere",
                    preamble
                )
            }
            "perplexity" => {
                delegate_prompt!(
                    perplexity::Client<HttpClient>,
                    "PERPLEXITY_API_KEY",
                    "Perplexity",
                    preamble
                )
            }
            "mistral" => {
                delegate_prompt!(
                    mistral::Client<HttpClient>,
                    "MISTRAL_API_KEY",
                    "Mistral",
                    preamble
                )
            }
            "groq" => delegate_prompt!(groq::Client<HttpClient>, "GROQ_API_KEY", "Groq", preamble),
            "xai" => delegate_prompt!(xai::Client<HttpClient>, "XAI_API_KEY", "xAI", preamble),
            _ => {
                let msg = format!("Unsupported AI provider: {}", provider);
                error!("{}", msg);
                return Err(AiServiceError::UnsupportedProvider(provider));
            }
        };

        Ok(response)
    }

    /// Predicts which backend should handle the request based on content.
    ///
    /// # Arguments
    ///
    /// * `request_info` - Description of the request (method, path, headers, body preview)
    /// * `backends` - List of available backends
    /// * `provider` - Optional override for AI provider
    /// * `model` - Optional override for AI model
    ///
    /// # Returns
    ///
    /// Index of the selected backend
    pub async fn predict_backend(
        &self,
        request_info: &str,
        backends: &[Backend],
        provider: Option<&str>,
        model: Option<&str>,
    ) -> Result<usize, PredictBackendError> {
        // Format backends list
        let backends_list = backends
            .iter()
            .enumerate()
            .map(|(i, b)| format!("{}: {} (port {})", i, b.host, b.port))
            .collect::<Vec<_>>()
            .join("\n");

        // Sanitize request info to prevent prompt injection
        let safe_request_info = request_info
            .replace("[REQUEST_START]", "")
            .replace("[REQUEST_END]", "");

        let prompt = format!(
            "Analyze this HTTP request and select the most appropriate backend service index.\n\n\
            [REQUEST_START]\n{}\n[REQUEST_END]\n\n\
            Available Backends:\n{}\n\n\
            Task: Return ONLY a valid JSON object with a single field 'index' containing the integer index (0 to {}) of the best backend. Do not include any markdown formatting, explanation, or extra text.",
            safe_request_info,
            backends_list,
            backends.len().saturating_sub(1)
        );

        let preamble = "You are an intelligent API gateway routing engine. Your job is to strictly analyze request content and route it to the correct backend service.";

        let response = self
            .execute_prompt(&prompt, preamble, provider, model)
            .await?;

        // Robust parsing using regex to find JSON or direct integer
        use regex::Regex;

        // Try to find {"index": N} pattern first
        let re_json = Regex::new(r#""index"\s*:\s*(\d+)"#).unwrap();
        if let Some(caps) = re_json.captures(&response) {
            if let Ok(index) = caps[1].parse::<usize>() {
                if index < backends.len() {
                    return Ok(index);
                }
            }
        }

        // Fallback: look for just a number
        let re_num = Regex::new(r"\b(\d+)\b").unwrap();
        if let Some(caps) = re_num.captures(&response) {
            if let Ok(index) = caps[1].parse::<usize>() {
                if index < backends.len() {
                    return Ok(index);
                }
            }
        }

        Err(PredictBackendError::InvalidIndex(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_settings_initialization() {
        let settings = AiSettings {
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            api_key: Some("test-key".to_string()),
        };

        let service = AiService::new(settings);
        assert_eq!(service.settings.provider, "openai");
        assert_eq!(service.settings.model, "gpt-4");
        assert_eq!(service.settings.api_key, Some("test-key".to_string()));
    }
}
