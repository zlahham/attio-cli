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
