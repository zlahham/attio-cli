use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ListNotesResponse {
    pub data: Vec<Note>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentifyResponse {
    pub active: bool,
    pub workspace_id: Option<String>,
    pub workspace_name: Option<String>,
    pub workspace_slug: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetNoteResponse {
    pub data: Note,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNoteRequest {
    pub data: CreateNoteData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNoteData {
    pub parent_object: String,
    pub parent_record_id: String,
    pub title: String,
    pub format: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: NoteId,
    pub parent_object: String,
    pub parent_record_id: String,
    pub title: String,
    pub content_plaintext: String,
    pub content_markdown: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoteId {
    pub workspace_id: String,
    pub note_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_note() {
        let json = r#"
        {
            "id": {
                "workspace_id": "ws_123",
                "note_id": "note_456"
            },
            "parent_object": "people",
            "parent_record_id": "00000000-0000-0000-0000-000000000000",
            "title": "Test Note",
            "content_plaintext": "Hello world",
            "content_markdown": "Hello **world**",
            "created_at": "2023-01-01T00:00:00Z"
        }
        "#;
        let note: Note = serde_json::from_str(json).unwrap();
        assert_eq!(note.title, "Test Note");
    }
}
