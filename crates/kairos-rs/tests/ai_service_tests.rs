use kairos_rs::services::ai::AiService;
use kairos_rs::models::settings::AiSettings;

#[test]
fn test_ai_settings_initialization() {
    let settings = AiSettings {
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
        api_key: Some("test-key".to_string()),
        fallback_providers: Vec::new(),
    };

    let service = AiService::new(settings);
    assert_eq!(service.settings.provider, "openai");
    assert_eq!(service.settings.model, "gpt-4");
    assert_eq!(service.settings.api_key, Some("test-key".to_string()));
    assert!(service.settings.fallback_providers.is_empty());
}
