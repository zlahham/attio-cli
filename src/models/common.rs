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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestItem {
        id: String,
        name: String,
    }

    #[test]
    fn test_list_response_serialization() {
        let items = vec![
            TestItem {
                id: "1".to_string(),
                name: "Item 1".to_string(),
            },
            TestItem {
                id: "2".to_string(),
                name: "Item 2".to_string(),
            },
        ];

        let response = ListResponse { data: items };
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ListResponse<TestItem> = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.data.len(), 2);
        assert_eq!(deserialized.data[0].id, "1");
        assert_eq!(deserialized.data[1].name, "Item 2");
    }

    #[test]
    fn test_get_response_serialization() {
        let item = TestItem {
            id: "123".to_string(),
            name: "Test".to_string(),
        };

        let response = GetResponse { data: item };
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: GetResponse<TestItem> = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.data.id, "123");
        assert_eq!(deserialized.data.name, "Test");
    }

    #[test]
    fn test_create_request_serialization() {
        let item = TestItem {
            id: "new".to_string(),
            name: "New Item".to_string(),
        };

        let request = CreateRequest { data: item };
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains("\"id\":\"new\""));
        assert!(json.contains("\"name\":\"New Item\""));
    }

    #[test]
    fn test_identify_response_deserialization() {
        let json = r#"{
            "active": true,
            "workspace_id": "ws_123",
            "workspace_name": "My Workspace",
            "workspace_slug": "my-workspace"
        }"#;

        let response: IdentifyResponse = serde_json::from_str(json).unwrap();

        assert!(response.active);
        assert_eq!(response.workspace_id, Some("ws_123".to_string()));
        assert_eq!(response.workspace_name, Some("My Workspace".to_string()));
        assert_eq!(response.workspace_slug, Some("my-workspace".to_string()));
    }

    #[test]
    fn test_identify_response_with_missing_optional_fields() {
        let json = r#"{"active": false}"#;
        let response: IdentifyResponse = serde_json::from_str(json).unwrap();

        assert!(!response.active);
        assert_eq!(response.workspace_id, None);
        assert_eq!(response.workspace_name, None);
        assert_eq!(response.workspace_slug, None);
    }
}
