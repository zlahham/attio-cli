use crate::models::ListNotesResponse;
use reqwest::{Client, header};
use std::error::Error;

const BASE_URL: &str = "https://api.attio.com/v2";

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
            return Err(format!("API Error ({}): {}", status, body).into());
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
            url.push_str("?");
            url.push_str(&query_params.join("&"));
        }

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(format!("API Error ({}): {}", status, body).into());
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
            return Err(format!("API Error ({}): {}", status, body).into());
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
            return Err(format!("API Error ({}): {}", status, body).into());
        }

        let response_data = response.json::<crate::models::GetNoteResponse>().await?;
        Ok(response_data)
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
            return Err(format!("API Error ({}): {}", status, body).into());
        }

        Ok(())
    }
}
