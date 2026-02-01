use crate::models::Note;

/// Estimates the memory size of a note in bytes.
///
/// This calculates the heap-allocated size (String contents) plus
/// the stack size of the Note struct itself.
pub fn estimate_note_size(note: &Note) -> usize {
    std::mem::size_of::<Note>()
        + note.id.workspace_id.capacity()
        + note.id.note_id.capacity()
        + note.parent_object.capacity()
        + note.parent_record_id.capacity()
        + note.title.capacity()
        + note.content_plaintext.capacity()
        + note.content_markdown.capacity()
        + note.created_at.capacity()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Note, NoteId};

    #[test]
    fn test_estimate_note_size() {
        let note = Note {
            id: NoteId {
                workspace_id: "ws_123".to_string(),
                note_id: "note_456".to_string(),
            },
            parent_object: "people".to_string(),
            parent_record_id: "00000000-0000-0000-0000-000000000000".to_string(),
            title: "Test Note".to_string(),
            content_plaintext: "Hello world".to_string(),
            content_markdown: "Hello **world**".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
        };

        let size = estimate_note_size(&note);
        // Size should be at least the size of the struct itself
        assert!(size >= std::mem::size_of::<Note>());
        // And should include some string data
        assert!(size > std::mem::size_of::<Note>());
    }
}
