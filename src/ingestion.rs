use uuid::Uuid;
use std::path::PathBuf;
use std::time::SystemTime;

struct Document {
    id: Uuid,
    path: PathBuf,
    content: String,
    modified: Option<SystemTime>,
}

#[derive(Debug)]
pub enum IngestError {
    NotDirectory,
    Io(std::io::Error), // Wrapping std::io::Error allows us to keep the full error history and can show the user what exactly went wrong
}