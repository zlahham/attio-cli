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
        assert_eq!(note.content_plaintext, "Hello world");
        assert_eq!(note.parent_object, "people");
    }

    #[test]
    fn test_note_id_deserialization() {
        let json = r#"{"workspace_id": "ws_abc", "note_id": "note_xyz"}"#;
        let note_id: NoteId = serde_json::from_str(json).unwrap();

        assert_eq!(note_id.workspace_id, "ws_abc");
        assert_eq!(note_id.note_id, "note_xyz");
    }

    #[test]
    fn test_create_note_data_serialization() {
        let data = CreateNoteData {
            parent_object: "companies".to_string(),
            parent_record_id: "comp_123".to_string(),
            title: "Meeting Notes".to_string(),
            format: "markdown".to_string(),
            content: "# Meeting Summary".to_string(),
        };

        let json = serde_json::to_string(&data).unwrap();

        assert!(json.contains("\"parent_object\":\"companies\""));
        assert!(json.contains("\"title\":\"Meeting Notes\""));
        assert!(json.contains("\"format\":\"markdown\""));
    }

    #[test]
    fn test_note_estimate_size_bytes() {
        let note = Note {
            id: NoteId {
                workspace_id: "ws_123".to_string(),
                note_id: "note_456".to_string(),
            },
            parent_object: "people".to_string(),
            parent_record_id: "rec_789".to_string(),
            title: "Test".to_string(),
            content_plaintext: "Content".to_string(),
            content_markdown: "**Content**".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
        };

        let size = note.estimate_size_bytes();

        // Size should be struct size + all string capacities
        assert!(size > 0);
        assert!(size >= std::mem::size_of::<Note>());
    }

    #[test]
    fn test_cacheable_trait_implementation() {
        // Test that Note implements Cacheable
        fn assert_cacheable<T: Cacheable>(_: &T) {}

        let note = Note {
            id: NoteId {
                workspace_id: "ws".to_string(),
                note_id: "note".to_string(),
            },
            parent_object: "test".to_string(),
            parent_record_id: "rec".to_string(),
            title: "Title".to_string(),
            content_plaintext: "Plain".to_string(),
            content_markdown: "Mark".to_string(),
            created_at: "2023".to_string(),
        };

        assert_cacheable(&note);
    }
}
