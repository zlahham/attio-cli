use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub token: String,
    #[serde(default = "default_cache_limit_mb")]
    pub cache_limit_mb: u64,
}

fn default_cache_limit_mb() -> u64 {
    50
}

impl Config {
    pub fn new(token: String) -> Self {
        Self {
            token,
            cache_limit_mb: default_cache_limit_mb(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new_uses_defaults() {
        let config = Config::new("test_token".to_string());
        assert_eq!(config.token, "test_token");
        assert_eq!(config.cache_limit_mb, 50);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            token: "my_token".to_string(),
            cache_limit_mb: 100,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.token, "my_token");
        assert_eq!(deserialized.cache_limit_mb, 100);
    }

    #[test]
    fn test_config_backward_compatibility_missing_cache_limit() {
        // Old format: just token field
        let json = r#"{"token": "old_token"}"#;
        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.token, "old_token");
        assert_eq!(config.cache_limit_mb, 50); // Should use default
    }

    #[test]
    fn test_config_with_custom_cache_limit() {
        let json = r#"{"token": "test", "cache_limit_mb": 200}"#;
        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.cache_limit_mb, 200);
    }
}
