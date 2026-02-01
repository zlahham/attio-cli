use serde::{Deserialize, Serialize};

/// Trait for models that can be cached with memory tracking
#[allow(dead_code)]
pub trait Cacheable {
    /// Estimate the memory size of this item in bytes
    fn estimate_size_bytes(&self) -> usize;
}

/// Generic wrapper for list/paginated responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
}

/// Generic wrapper for single item responses
#[derive(Debug, Serialize, Deserialize)]
pub struct GetResponse<T> {
    pub data: T,
}

/// Generic wrapper for create requests
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRequest<T> {
    pub data: T,
}

/// Response from the identify/self endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct IdentifyResponse {
    pub active: bool,
    pub workspace_id: Option<String>,
    pub workspace_name: Option<String>,
    pub workspace_slug: Option<String>,
}
