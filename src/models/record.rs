use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::common::Cacheable;

/// Request body data for creating or updating a record.
/// Wraps a `values` map where keys are attribute slugs (e.g. "name", "domains")
/// and values are the attribute values in Attio's expected format.
///
/// Used inside `CreateRequest<CreateOrUpdateRecordData>` which serializes to:
/// `{ "data": { "values": { "attr": [...] } } }`
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrUpdateRecordData {
    pub values: HashMap<String, serde_json::Value>,
}

/// Unique identifier for a record in Attio.
/// Records are identified by a combination of workspace, object, and record IDs.
#[derive(Debug, Serialize, Deserialize)]
pub struct RecordId {
    pub workspace_id: String,
    pub object_id: String,
    pub record_id: String,
}

/// A record from the Attio records API.
///
/// Records are the core data objects in Attio (companies, people, deals, etc.).
/// The `values` field is dynamic — each attribute key maps to an array of value
/// objects whose shape depends on the attribute type.
#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub id: RecordId,
    pub created_at: String,
    pub web_url: String,
    pub values: HashMap<String, Vec<serde_json::Value>>,
}

/// Request body for the POST /v2/objects/{object}/records/query endpoint.
#[derive(Debug, Serialize, Deserialize)]
pub struct RecordQueryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

/// Known fields where Attio stores the display value for different attribute types.
/// The helper tries each field in order and returns the first match.
const VALUE_FIELDS: &[&str] = &[
    "value",
    "domain",
    "email_address",
    "phone_number",
    "locality",
    "title",
    "full_name",
];

impl Record {
    /// Extract the first display value for a given attribute key.
    ///
    /// Attio stores attribute values as arrays of objects. Different attribute types
    /// put the "display" value in different JSON fields. This method looks at the
    /// first element in the array and tries known fields in order.
    ///
    /// For example, a "name" attribute might look like:
    /// ```json
    /// "name": [{ "value": "Acme Corp", "attribute_type": "text" }]
    /// ```
    /// While a "domains" attribute might look like:
    /// ```json
    /// "domains": [{ "domain": "acme.com", "attribute_type": "domain" }]
    /// ```
    pub fn extract_first_value(&self, key: &str) -> Option<String> {
        let values = self.values.get(key)?;
        let first = values.first()?;
        let obj = first.as_object()?;

        for &field in VALUE_FIELDS {
            if let Some(val) = obj.get(field) {
                if let Some(s) = val.as_str() {
                    if !s.is_empty() {
                        return Some(s.to_string());
                    }
                }
            }
        }

        None
    }

    /// Get a human-readable display name for this record.
    /// Tries the "name" attribute first, falling back to "(unnamed)".
    pub fn display_name(&self) -> String {
        self.extract_first_value("name")
            .unwrap_or_else(|| "(unnamed)".to_string())
    }

    /// Collect all attributes that have a displayable value, sorted by key.
    /// Useful for rendering a detail view of a record.
    pub fn all_display_values(&self) -> Vec<(String, String)> {
        let mut pairs: Vec<(String, String)> = self
            .values
            .keys()
            .filter_map(|key| {
                self.extract_first_value(key)
                    .map(|val| (key.clone(), val))
            })
            .collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        pairs
    }
}

impl Cacheable for Record {
    fn estimate_size_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.id.workspace_id.capacity()
            + self.id.object_id.capacity()
            + self.id.record_id.capacity()
            + self.created_at.capacity()
            + self.web_url.capacity()
            + self
                .values
                .iter()
                .map(|(k, v)| k.capacity() + serde_json::to_string(v).unwrap_or_default().len())
                .sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a Record from a JSON string for the values field.
    fn record_with_values(values_json: &str) -> Record {
        let values: HashMap<String, Vec<serde_json::Value>> =
            serde_json::from_str(values_json).unwrap();
        Record {
            id: RecordId {
                workspace_id: "ws_1".to_string(),
                object_id: "obj_1".to_string(),
                record_id: "rec_1".to_string(),
            },
            created_at: "2024-01-01T00:00:00Z".to_string(),
            web_url: "https://app.attio.com/test/company/rec_1".to_string(),
            values,
        }
    }

    #[test]
    fn test_deserialize_full_record() {
        let json = r#"{
            "id": {
                "workspace_id": "ws_abc",
                "object_id": "obj_companies",
                "record_id": "rec_xyz"
            },
            "created_at": "2024-06-01T12:00:00Z",
            "web_url": "https://app.attio.com/myworkspace/company/rec_xyz",
            "values": {
                "name": [{"value": "Acme Corp", "attribute_type": "text"}],
                "domains": [{"domain": "acme.com", "attribute_type": "domain"}]
            }
        }"#;

        let record: Record = serde_json::from_str(json).unwrap();

        assert_eq!(record.id.workspace_id, "ws_abc");
        assert_eq!(record.id.object_id, "obj_companies");
        assert_eq!(record.id.record_id, "rec_xyz");
        assert_eq!(record.created_at, "2024-06-01T12:00:00Z");
        assert_eq!(record.values.len(), 2);
    }

    #[test]
    fn test_record_id_deserialization() {
        let json = r#"{
            "workspace_id": "ws_1",
            "object_id": "obj_2",
            "record_id": "rec_3"
        }"#;
        let id: RecordId = serde_json::from_str(json).unwrap();
        assert_eq!(id.workspace_id, "ws_1");
        assert_eq!(id.object_id, "obj_2");
        assert_eq!(id.record_id, "rec_3");
    }

    #[test]
    fn test_extract_text_value() {
        let record = record_with_values(r#"{"name": [{"value": "Acme Corp"}]}"#);
        assert_eq!(
            record.extract_first_value("name"),
            Some("Acme Corp".to_string())
        );
    }

    #[test]
    fn test_extract_domain_value() {
        let record = record_with_values(r#"{"domains": [{"domain": "acme.com"}]}"#);
        assert_eq!(
            record.extract_first_value("domains"),
            Some("acme.com".to_string())
        );
    }

    #[test]
    fn test_extract_email_value() {
        let record =
            record_with_values(r#"{"email_addresses": [{"email_address": "hi@acme.com"}]}"#);
        assert_eq!(
            record.extract_first_value("email_addresses"),
            Some("hi@acme.com".to_string())
        );
    }

    #[test]
    fn test_extract_phone_value() {
        let record =
            record_with_values(r#"{"phone_numbers": [{"phone_number": "+1-555-0100"}]}"#);
        assert_eq!(
            record.extract_first_value("phone_numbers"),
            Some("+1-555-0100".to_string())
        );
    }

    #[test]
    fn test_extract_location_value() {
        let record =
            record_with_values(r#"{"primary_location": [{"locality": "San Francisco"}]}"#);
        assert_eq!(
            record.extract_first_value("primary_location"),
            Some("San Francisco".to_string())
        );
    }

    #[test]
    fn test_extract_title_value() {
        let record = record_with_values(r#"{"job_title": [{"title": "CEO"}]}"#);
        assert_eq!(
            record.extract_first_value("job_title"),
            Some("CEO".to_string())
        );
    }

    #[test]
    fn test_extract_full_name_value() {
        let record = record_with_values(r#"{"name": [{"full_name": "Ada Lovelace"}]}"#);
        assert_eq!(
            record.extract_first_value("name"),
            Some("Ada Lovelace".to_string())
        );
    }

    #[test]
    fn test_extract_missing_key_returns_none() {
        let record = record_with_values(r#"{"name": [{"value": "Test"}]}"#);
        assert_eq!(record.extract_first_value("nonexistent"), None);
    }

    #[test]
    fn test_extract_empty_array_returns_none() {
        let record = record_with_values(r#"{"name": []}"#);
        assert_eq!(record.extract_first_value("name"), None);
    }

    #[test]
    fn test_extract_empty_string_skipped() {
        // If "value" is empty, should return None (no fallback fields match either)
        let record = record_with_values(r#"{"name": [{"value": ""}]}"#);
        assert_eq!(record.extract_first_value("name"), None);
    }

    #[test]
    fn test_display_name_with_name() {
        let record = record_with_values(r#"{"name": [{"value": "Acme Corp"}]}"#);
        assert_eq!(record.display_name(), "Acme Corp");
    }

    #[test]
    fn test_display_name_unnamed_fallback() {
        let record = record_with_values(r#"{"domains": [{"domain": "acme.com"}]}"#);
        assert_eq!(record.display_name(), "(unnamed)");
    }

    #[test]
    fn test_all_display_values_sorted() {
        let record = record_with_values(
            r#"{
                "name": [{"value": "Acme Corp"}],
                "domains": [{"domain": "acme.com"}],
                "description": [{"value": "A company"}]
            }"#,
        );
        let pairs = record.all_display_values();
        assert_eq!(pairs.len(), 3);
        // Should be sorted alphabetically by key
        assert_eq!(pairs[0].0, "description");
        assert_eq!(pairs[1].0, "domains");
        assert_eq!(pairs[2].0, "name");
    }

    #[test]
    fn test_all_display_values_skips_unextractable() {
        let record = record_with_values(
            r#"{
                "name": [{"value": "Test"}],
                "weird_attr": [{"unknown_field": 42}]
            }"#,
        );
        let pairs = record.all_display_values();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "name");
    }

    #[test]
    fn test_record_query_request_serialization_empty() {
        let req = RecordQueryRequest {
            limit: None,
            offset: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_record_query_request_serialization_with_values() {
        let req = RecordQueryRequest {
            limit: Some(20),
            offset: Some(10),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"limit\":20"));
        assert!(json.contains("\"offset\":10"));
    }

    #[test]
    fn test_cacheable_implementation() {
        let record = record_with_values(r#"{"name": [{"value": "Test"}]}"#);
        let size = record.estimate_size_bytes();
        assert!(size > 0);
        assert!(size >= std::mem::size_of::<Record>());
    }

    #[test]
    fn test_create_or_update_record_data_serialization() {
        let mut values = HashMap::new();
        values.insert(
            "name".to_string(),
            serde_json::json!([{"value": "Acme Corp"}]),
        );
        values.insert(
            "domains".to_string(),
            serde_json::json!([{"domain": "acme.com"}]),
        );

        let data = CreateOrUpdateRecordData { values };
        let json = serde_json::to_value(&data).unwrap();

        let values_obj = json.get("values").unwrap().as_object().unwrap();
        assert_eq!(
            values_obj.get("name").unwrap(),
            &serde_json::json!([{"value": "Acme Corp"}])
        );
        assert_eq!(
            values_obj.get("domains").unwrap(),
            &serde_json::json!([{"domain": "acme.com"}])
        );
    }

    #[test]
    fn test_create_or_update_record_data_wrapped_in_create_request() {
        use crate::models::CreateRequest;

        let mut values = HashMap::new();
        values.insert(
            "name".to_string(),
            serde_json::json!([{"value": "Test Co"}]),
        );

        let request = CreateRequest {
            data: CreateOrUpdateRecordData { values },
        };
        let json = serde_json::to_value(&request).unwrap();

        // Should serialize as { "data": { "values": { ... } } }
        let data = json.get("data").unwrap();
        let values_obj = data.get("values").unwrap().as_object().unwrap();
        assert_eq!(
            values_obj.get("name").unwrap(),
            &serde_json::json!([{"value": "Test Co"}])
        );
    }

    #[test]
    fn test_create_or_update_record_data_empty_values() {
        let data = CreateOrUpdateRecordData {
            values: HashMap::new(),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert_eq!(json, r#"{"values":{}}"#);
    }
}
