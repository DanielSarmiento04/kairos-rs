use crate::models::settings::AiSettings;
use crate::models::router::Backend;
use log::{error, debug};
use reqwest::Client as HttpClient;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::{anthropic, cohere, groq, mistral, openai, perplexity, xai};

/// Service for handling AI-related operations.
pub struct AiService {
    settings: AiSettings,
}

impl AiService {
    /// Creates a new instance of AiService.
    pub fn new(settings: AiSettings) -> Self {
        Self { settings }
    }

    /// Performs a simple completion request to test the integration.
    pub async fn ask(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let provider = self.settings.provider.to_lowercase();
        let model = &self.settings.model;

        // Common preamble for the agent
        let preamble = "You are a helpful AI assistant integrated into the Kairos-rs gateway.";

        // Helper to get API key from config or env
        let get_key = |env_var: &str| -> Result<String, Box<dyn std::error::Error>> {
            self.settings
                .api_key
                .clone()
                .or_else(|| std::env::var(env_var).ok())
                .ok_or_else(|| format!("API key not found in config or {} env var", env_var).into())
        };

        let response = {
            macro_rules! delegate_prompt {
                ($client:ty, $key_env:literal, $provider_name:literal, $preamble:expr) => {{
                    let client = <$client>::new(&get_key($key_env)?)?;
                    let agent = client.agent(model).preamble($preamble).build();
                    debug!("Sending prompt to {} model: {}", $provider_name, model);
                    agent.prompt(prompt).await?
                }};
            }

            match provider.as_str() {
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
    /// 
    /// # Returns
    /// 
    /// Index of the selected backend
    pub async fn predict_backend(&self, request_info: &str, backends: &[Backend]) -> Result<usize, Box<dyn std::error::Error>> {
        // Format backends list
        let backends_list = backends.iter().enumerate()
            .map(|(i, b)| format!("{}: {} (port {})", i, b.host, b.port))
            .collect::<Vec<_>>()
            .join("\n");
            
        let prompt = format!(
            "Analyze this HTTP request and select the most appropriate backend service index.\n\n\
            Request:\n{}\n\n\
            Available Backends:\n{}\n\n\
            Task: Return ONLY the index number (0 to {}) of the best backend. Do not include any explanation or extra text.",
            request_info,
            backends_list,
            backends.len().saturating_sub(1)
        );
        
        // Use a more specific preamble for this task
        let provider = self.settings.provider.to_lowercase();
        let model = &self.settings.model;
        
        let preamble = "You are an intelligent API gateway routing engine. Your job is to strictly analyze request content and route it to the correct backend service.";
        
        let get_key = |env_var: &str| -> Result<String, Box<dyn std::error::Error>> {
            self.settings
                .api_key
                .clone()
                .or_else(|| std::env::var(env_var).ok())
                .ok_or_else(|| format!("API key not found in config or {} env var", env_var).into())
        };

        let response = {
            macro_rules! delegate_prompt {
                ($client:ty, $key_env:literal, $provider_name:literal, $preamble:expr) => {{
                    let client = <$client>::new(&get_key($key_env)?)?;
                    let agent = client.agent(model).preamble($preamble).build();
                    debug!("Sending routing prompt to {} model: {}", $provider_name, model);
                    agent.prompt(&prompt).await?
                }};
            }
            
            // Re-using the same match logic but with the routing preamble
             match provider.as_str() {
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
                    return Err(format!("Unsupported AI provider: {}", provider).into());
                }
            }
        };
        
        // Parse the response to get the index
        let response_trimmed = response.trim();
        // Try to find a number in the response if it's "Index: 0" or "The index is 0"
        let index_str = response_trimmed.chars()
            .skip_while(|c| !c.is_digit(10))
            .take_while(|c| c.is_digit(10))
            .collect::<String>();
            
        if let Ok(index) = index_str.parse::<usize>() {
            if index < backends.len() {
                return Ok(index);
            }
        }
        
        Err(format!("AI returned invalid backend index: '{}'", response_trimmed).into())
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
