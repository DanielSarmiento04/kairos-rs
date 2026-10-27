use kairos_rs::models::settings::{AiProviderConfig, AiSettings};
use kairos_rs::services::ai::{AiService, AiServiceError};

#[test]
fn test_ai_settings_initialization() {
    let settings = AiSettings {
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
        api_key: Some("test-key".to_string()),
        fallback_providers: vec![AiProviderConfig {
            provider: "anthropic".to_string(),
            model: "claude-3-5-sonnet".to_string(),
            api_key: Some("fallback-key".to_string()),
        }],
        semantic_cache: None,
    };

    let service = AiService::new(settings);
    assert_eq!(service.settings.provider, "openai");
    assert_eq!(service.settings.model, "gpt-4");
    assert_eq!(service.settings.api_key, Some("test-key".to_string()));
    assert_eq!(service.settings.fallback_providers.len(), 1);
    assert_eq!(service.settings.fallback_providers[0].provider, "anthropic");
}

#[tokio::test]
async fn test_unsupported_provider() {
    let settings = AiSettings {
        provider: "unknown-vendor".to_string(),
        model: "model-x".to_string(),
        api_key: Some("key".to_string()),
        fallback_providers: vec![],
        semantic_cache: None,
    };
    let service = AiService::new(settings);
    let res = service.ask("hello").await;
    assert!(matches!(res, Err(AiServiceError::UnsupportedProvider(_))));
}

#[tokio::test]
async fn test_api_key_not_found() {
    let settings = AiSettings {
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
        api_key: None,
        fallback_providers: vec![],
        semantic_cache: None,
    };
    // Ensure env var is not set during test
    std::env::remove_var("OPENAI_API_KEY");
    let service = AiService::new(settings);
    let res = service
        .execute_single_provider("hello", "preamble", "openai", "gpt-4", None)
        .await;
    assert!(matches!(res, Err(AiServiceError::ApiKeyNotFound(_))));
}

#[tokio::test]
async fn test_generate_embedding_local_fallback() {
    let settings = AiSettings {
        provider: "local".to_string(),
        model: "default".to_string(),
        api_key: None,
        fallback_providers: vec![],
        semantic_cache: None,
    };
    let service = AiService::new(settings);
    let vec = service
        .generate_embedding("How do I sort an array?", None, None, None)
        .await
        .unwrap();
    assert_eq!(vec.len(), 128);
}
