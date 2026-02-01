use serde::{Deserialize, Serialize};

use super::common::Cacheable;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNoteData {
    pub parent_object: String,
    pub parent_record_id: String,
    pub title: String,
    pub format: String,
    pub content: String,
}

impl Cacheable for Note {
    /// Estimate the memory size of this note in bytes
    fn estimate_size_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.id.workspace_id.capacity()
            + self.id.note_id.capacity()
            + self.parent_object.capacity()
            + self.parent_record_id.capacity()
            + self.title.capacity()
            + self.content_plaintext.capacity()
            + self.content_markdown.capacity()
            + self.created_at.capacity()
    }
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
