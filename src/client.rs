use crate::models::{ApiErrorBody, ListNotesResponse, ListRecordsResponse, RecordQueryRequest};
use reqwest::{Client, StatusCode, header};
use std::error::Error;

const BASE_URL: &str = "https://api.attio.com/v2";

/// Try to extract a human-readable message from an Attio API error response.
/// Falls back to the raw body if it can't be parsed.
fn format_api_error(status: StatusCode, body: &str) -> String {
    if let Ok(api_error) = serde_json::from_str::<ApiErrorBody>(body) {
        format!("API Error ({}): {}", status, api_error.message)
    } else {
        format!("API Error ({}): {}", status, body)
    }
}

pub struct AttioClient {
    client: Client,
}

impl AttioClient {
    pub fn new(token: String) -> Self {
        let mut headers = header::HeaderMap::new();

        let mut auth_value = header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap();
        auth_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_value);
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("attio-cli/0.1.0"),
        );

        let client = Client::builder().default_headers(headers).build().unwrap();

        Self { client }
    }

    pub async fn identify(&self) -> Result<crate::models::IdentifyResponse, Box<dyn Error>> {
        let response = self.client.get(format!("{}/self", BASE_URL)).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(format_api_error(status, &body).into());
        }

        let response_data = response.json::<crate::models::IdentifyResponse>().await?;
        Ok(response_data)
    }

    pub async fn list_notes(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<ListNotesResponse, Box<dyn Error>> {
        let mut url = format!("{}/notes", BASE_URL);
        let mut query_params = Vec::new();

        if let Some(limit) = limit {
            query_params.push(format!("limit={}", limit));
        }
        if let Some(offset) = offset {
            query_params.push(format!("offset={}", offset));
        }

        if !query_params.is_empty() {
            url.push('?');
            url.push_str(&query_params.join("&"));
        }

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(format_api_error(status, &body).into());
        }

        let body = response.text().await?;
        let response_data = serde_json::from_str::<ListNotesResponse>(&body)?;
        Ok(response_data)
    }

    pub async fn get_note(
        &self,
        note_id: &str,
    ) -> Result<crate::models::GetNoteResponse, Box<dyn Error>> {
        let response = self
            .client
            .get(format!("{}/notes/{}", BASE_URL, note_id))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(format_api_error(status, &body).into());
        }

        let response_data = response.json::<crate::models::GetNoteResponse>().await?;
        Ok(response_data)
    }

    pub async fn create_note(
        &self,
        data: crate::models::CreateNoteRequest,
    ) -> Result<crate::models::GetNoteResponse, Box<dyn Error>> {
        let response = self
            .client
            .post(format!("{}/notes", BASE_URL))
            .json(&data)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(format_api_error(status, &body).into());
        }

        let response_data = response.json::<crate::models::GetNoteResponse>().await?;
        Ok(response_data)
    }

    pub async fn list_records(
        &self,
        object: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<ListRecordsResponse, Box<dyn Error>> {
        let url = format!("{}/objects/{}/records/query", BASE_URL, object);
        let body = RecordQueryRequest { limit, offset };

        let response = self.client.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(format_api_error(status, &body).into());
        }

        let response_data = response.json::<ListRecordsResponse>().await?;
        Ok(response_data)
    }

    pub async fn get_record(
        &self,
        object: &str,
        record_id: &str,
    ) -> Result<crate::models::GetRecordResponse, Box<dyn Error>> {
        let url = format!("{}/objects/{}/records/{}", BASE_URL, object, record_id);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(format_api_error(status, &body).into());
        }

        let response_data = response.json::<crate::models::GetRecordResponse>().await?;
        Ok(response_data)
    }

    pub async fn create_record(
        &self,
        object: &str,
        data: crate::models::CreateOrUpdateRecordRequest,
    ) -> Result<crate::models::GetRecordResponse, Box<dyn Error>> {
        let url = format!("{}/objects/{}/records", BASE_URL, object);
        let response = self.client.post(&url).json(&data).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(format_api_error(status, &body).into());
        }

        let response_data = response.json::<crate::models::GetRecordResponse>().await?;
        Ok(response_data)
    }

    pub async fn update_record(
        &self,
        object: &str,
        record_id: &str,
        data: crate::models::CreateOrUpdateRecordRequest,
    ) -> Result<crate::models::GetRecordResponse, Box<dyn Error>> {
        let url = format!("{}/objects/{}/records/{}", BASE_URL, object, record_id);
        let response = self.client.patch(&url).json(&data).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(format_api_error(status, &body).into());
        }

        let response_data = response.json::<crate::models::GetRecordResponse>().await?;
        Ok(response_data)
    }

    pub async fn delete_record(&self, object: &str, record_id: &str) -> Result<(), Box<dyn Error>> {
        let url = format!("{}/objects/{}/records/{}", BASE_URL, object, record_id);
        let response = self.client.delete(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(format_api_error(status, &body).into());
        }

        Ok(())
    }

    pub async fn delete_note(&self, note_id: &str) -> Result<(), Box<dyn Error>> {
        let response = self
            .client
            .delete(format!("{}/notes/{}", BASE_URL, note_id))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(format_api_error(status, &body).into());
        }

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn build_records_query_url(object: &str) -> String {
        format!("{}/objects/{}/records/query", BASE_URL, object)
    }

    #[cfg(test)]
    pub(crate) fn build_record_url(object: &str, record_id: &str) -> String {
        format!("{}/objects/{}/records/{}", BASE_URL, object, record_id)
    }

    #[cfg(test)]
    pub(crate) fn build_create_record_url(object: &str) -> String {
        format!("{}/objects/{}/records", BASE_URL, object)
    }

    #[cfg(test)]
    pub(crate) fn build_notes_url(limit: Option<u32>, offset: Option<u32>) -> String {
        let mut url = format!("{}/notes", BASE_URL);
        let mut query_params = Vec::new();

        if let Some(limit) = limit {
            query_params.push(format!("limit={}", limit));
        }
        if let Some(offset) = offset {
            query_params.push(format!("offset={}", offset));
        }

        if !query_params.is_empty() {
            url.push('?');
            url.push_str(&query_params.join("&"));
        }

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = AttioClient::new("test_token".to_string());
        // Just verify it doesn't panic
        assert!(std::mem::size_of_val(&client) > 0);
    }

    #[test]
    fn test_build_notes_url_no_params() {
        let url = AttioClient::build_notes_url(None, None);
        assert_eq!(url, "https://api.attio.com/v2/notes");
    }

    #[test]
    fn test_build_notes_url_with_limit() {
        let url = AttioClient::build_notes_url(Some(50), None);
        assert_eq!(url, "https://api.attio.com/v2/notes?limit=50");
    }

    #[test]
    fn test_build_notes_url_with_offset() {
        let url = AttioClient::build_notes_url(None, Some(100));
        assert_eq!(url, "https://api.attio.com/v2/notes?offset=100");
    }

    #[test]
    fn test_build_notes_url_with_both_params() {
        let url = AttioClient::build_notes_url(Some(25), Some(50));
        assert_eq!(url, "https://api.attio.com/v2/notes?limit=25&offset=50");
    }

    #[test]
    fn test_base_url_is_v2() {
        assert_eq!(BASE_URL, "https://api.attio.com/v2");
    }

    #[test]
    fn test_build_records_query_url() {
        let url = AttioClient::build_records_query_url("companies");
        assert_eq!(
            url,
            "https://api.attio.com/v2/objects/companies/records/query"
        );
    }

    #[test]
    fn test_build_record_url() {
        let url = AttioClient::build_record_url("companies", "rec_abc123");
        assert_eq!(
            url,
            "https://api.attio.com/v2/objects/companies/records/rec_abc123"
        );
    }

    #[test]
    fn test_build_records_query_url_people() {
        let url = AttioClient::build_records_query_url("people");
        assert_eq!(url, "https://api.attio.com/v2/objects/people/records/query");
    }

    #[test]
    fn test_build_create_record_url() {
        let url = AttioClient::build_create_record_url("companies");
        assert_eq!(url, "https://api.attio.com/v2/objects/companies/records");
    }

    #[test]
    fn test_build_create_record_url_people() {
        let url = AttioClient::build_create_record_url("people");
        assert_eq!(url, "https://api.attio.com/v2/objects/people/records");
    }

    #[test]
    fn test_format_api_error_with_valid_json() {
        let body = r#"{"status_code":404,"type":"invalid_request_error","code":"not_found","message":"Record with ID \"abc-123\" was not found."}"#;
        let result = format_api_error(StatusCode::NOT_FOUND, body);
        assert_eq!(
            result,
            r#"API Error (404 Not Found): Record with ID "abc-123" was not found."#
        );
    }

    #[test]
    fn test_format_api_error_with_invalid_json() {
        let body = "Something went wrong";
        let result = format_api_error(StatusCode::INTERNAL_SERVER_ERROR, body);
        assert_eq!(
            result,
            "API Error (500 Internal Server Error): Something went wrong"
        );
    }
}
