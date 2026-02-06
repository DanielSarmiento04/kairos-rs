use crate::models::settings::AiSettings;
use log::{error, info};
use rig::client::CompletionClient;
use rig::client::ProviderClient;
use rig::completion::Prompt;
use rig::providers::openai;

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

        match provider.as_str() {
            "openai" => {
                // Create OpenAI client from env (temporary test)
                let client = openai::Client::from_env();

                // Create agent with the configured model
                let agent = client
                    .agent(&self.settings.model)
                    .preamble(
                        "You are a helpful AI assistant integrated into the Kairos-rs gateway.",
                    )
                    .build();

                info!("Sending prompt to OpenAI model: {}", self.settings.model);

                // Send prompt
                let response = agent.prompt(prompt).await?;
                Ok(response)
            }
            _ => {
                let msg = format!("Unsupported AI provider: {}", provider);
                error!("{}", msg);
                Err(msg.into())
            }
        }
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
