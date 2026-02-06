use crate::models::settings::AiSettings;
use log::{error, info};
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

        let response = match provider.as_str() {
            "openai" => {
                let client = openai::Client::<HttpClient>::new(&get_key("OPENAI_API_KEY")?)?;
                let agent = client.agent(model).preamble(preamble).build();
                info!("Sending prompt to OpenAI model: {}", model);
                agent.prompt(prompt).await?
            }
            "anthropic" => {
                let client = anthropic::Client::<HttpClient>::new(&get_key("ANTHROPIC_API_KEY")?)?;
                let agent = client.agent(model).preamble(preamble).build();
                info!("Sending prompt to Anthropic model: {}", model);
                agent.prompt(prompt).await?
            }
            "cohere" => {
                let client = cohere::Client::<HttpClient>::new(&get_key("COHERE_API_KEY")?)?;
                let agent = client.agent(model).preamble(preamble).build();
                info!("Sending prompt to Cohere model: {}", model);
                agent.prompt(prompt).await?
            }
            "perplexity" => {
                let client =
                    perplexity::Client::<HttpClient>::new(&get_key("PERPLEXITY_API_KEY")?)?;
                let agent = client.agent(model).preamble(preamble).build();
                info!("Sending prompt to Perplexity model: {}", model);
                agent.prompt(prompt).await?
            }
            "mistral" => {
                let client = mistral::Client::<HttpClient>::new(&get_key("MISTRAL_API_KEY")?)?;
                let agent = client.agent(model).preamble(preamble).build();
                info!("Sending prompt to Mistral model: {}", model);
                agent.prompt(prompt).await?
            }
            "groq" => {
                let client = groq::Client::<HttpClient>::new(&get_key("GROQ_API_KEY")?)?;
                let agent = client.agent(model).preamble(preamble).build();
                info!("Sending prompt to Groq model: {}", model);
                agent.prompt(prompt).await?
            }
            "xai" => {
                let client = xai::Client::<HttpClient>::new(&get_key("XAI_API_KEY")?)?;
                let agent = client.agent(model).preamble(preamble).build();
                info!("Sending prompt to xAI model: {}", model);
                agent.prompt(prompt).await?
            }
            _ => {
                let msg = format!("Unsupported AI provider: {}", provider);
                error!("{}", msg);
                return Err(msg.into());
            }
        };

        Ok(response)
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
