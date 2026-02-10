use crate::models::settings::AiSettings;
use crate::models::router::Backend;
use log::{error, debug};
use reqwest::Client as HttpClient;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::{anthropic, cohere, groq, mistral, openai, perplexity, xai};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("API key not found for provider: {0}")]
    ApiKeyMissing(String),

    #[error("Unsupported AI provider: {0}")]
    UnsupportedProvider(String),

    #[error("AI request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    
    #[error("Error from AI provider: {0}")]
    ProviderError(String),
}

#[derive(Debug, Error)]
pub enum PredictBackendError {
    #[error("AI execution failed")]
    AiError(#[from] AiError),

    #[error("Failed to parse AI response: {0}")]
    ResponseParseError(String),

    #[error("AI returned an invalid or out-of-bounds backend index from response: {0}")]
    InvalidIndex(String),
}

/// Service for handling AI-related operations.
pub struct AiService {
    settings: AiSettings,
}

impl AiService {
    /// Creates a new instance of AiService.
    pub fn new(settings: AiSettings) -> Self {
        Self { settings }
    }

    /// Internal helper to execute a prompt against the configured provider
    async fn execute_prompt(
        &self, 
        prompt: &str, 
        preamble: &str,
        provider_override: Option<&str>,
        model_override: Option<&str>
    ) -> Result<String, Box<dyn std::error::Error>> {
        let provider = provider_override.unwrap_or(&self.settings.provider).to_lowercase();
        let model = model_override.unwrap_or(&self.settings.model);

         // Helper to get API key from config or env
         let get_key = |env_var: &str| -> Result<String, Box<dyn std::error::Error>> {
            self.settings
                .api_key
                .clone()
                .or_else(|| std::env::var(env_var).ok())
                .ok_or_else(|| format!("API key not found in config or {} env var", env_var).into())
        };

        macro_rules! delegate_prompt {
            ($client:ty, $key_env:literal, $provider_name:literal, $preamble:expr) => {{
                let client = <$client>::new(&get_key($key_env)?)?;
                let agent = client.agent(model).preamble($preamble).build();
                debug!("Sending prompt to {} model: {}", $provider_name, model);
                agent.prompt(prompt).await?
            }};
        }

        let response = match provider.as_str() {
            "openai" => {
                delegate_prompt!(openai::Client<HttpClient>, "OPENAI_API_KEY", "OpenAI", preamble)
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
                delegate_prompt!(cohere::Client<HttpClient>, "COHERE_API_KEY", "Cohere", preamble)
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
                delegate_prompt!(mistral::Client<HttpClient>, "MISTRAL_API_KEY", "Mistral", preamble)
            }
            "groq" => delegate_prompt!(groq::Client<HttpClient>, "GROQ_API_KEY", "Groq", preamble),
            "xai" => delegate_prompt!(xai::Client<HttpClient>, "XAI_API_KEY", "xAI", preamble),
            _ => {
                let msg = format!("Unsupported AI provider: {}", provider);
                error!("{}", msg);
                return Err(msg.into());
            }
        };

        Ok(response)
    }

    /// Performs a simple completion request to test the integration.
    pub async fn ask(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let preamble = "You are a helpful AI assistant integrated into the Kairos-rs gateway.";
        self.execute_prompt(prompt, preamble, None, None).await
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
        model: Option<&str>
    ) -> Result<usize, Box<dyn std::error::Error>> {
        // Format backends list
        let backends_list = backends.iter().enumerate()
            .map(|(i, b)| format!("{}: {} (port {})", i, b.host, b.port))
            .collect::<Vec<_>>()
            .join("\n");
            
        // Sanitize request info to prevent prompt injection
        let safe_request_info = request_info.replace("[REQUEST_START]", "").replace("[REQUEST_END]", "");

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

        let response = self.execute_prompt(&prompt, preamble, provider, model).await?;
        
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
        
        Err(format!("AI returned invalid or out-of-bounds backend index. Response: '{}'", response).into())
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
