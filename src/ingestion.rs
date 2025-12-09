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
    IoError(std::io::Error),
    InvalidData(String),
    NotDirectory(PathBuf),
}