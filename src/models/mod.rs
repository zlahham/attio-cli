// Common types and traits
mod common;
pub use common::*;

// Resource modules
pub mod config;
pub mod note;
pub mod record;

// Re-export commonly used types
pub use config::Config;
#[allow(unused_imports)]
pub use note::{CreateNoteData, Note, NoteId};
#[allow(unused_imports)]
pub use record::{Record, RecordId, RecordQueryRequest};

// Type aliases for backward compatibility and convenience
pub type ListNotesResponse = ListResponse<Note>;
pub type GetNoteResponse = GetResponse<Note>;
pub type CreateNoteRequest = CreateRequest<CreateNoteData>;
pub type ListRecordsResponse = ListResponse<Record>;
pub type GetRecordResponse = GetResponse<Record>;
