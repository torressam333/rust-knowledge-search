use uuid::Uuid;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct Document {
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

pub fn load_documents(dir: &Path) -> Result<Vec<Document>, IngestError> {
    // 1. Ensure the path is a directory
    if !dir.is_dir() {
        return Err(IngestError::NotDirectory);
    }

    // 2. Read directory entries
    let entries = std::fs::read_dir(dir).map_err(IngestError::Io)?;

    let mut docs = Vec::new();

    // 3. Process each entry
    for entry_result in entries {
        let entry = entry_result.map_err(IngestError::Io)?;
        let path = entry.path();

        // 4. Only allow .md or .txt files
        let ext = path.extension().and_then(|e| e.to_str());
        let is_text = matches!(ext, Some("md") | Some("txt"));
        if !is_text {
            continue;
        }

        // 5. Read the file contents
        let content = std::fs::read_to_string(&path).map_err(IngestError::Io)?;

        // 6. Get modified time (optional)
        let modified = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok();

        // 7. Build the Document
        docs.push(Document {
            id: Uuid::new_v4(),
            path,
            content,
            modified,
        });
    }

    Ok(docs)
}

#[cfg(test)]
mod tests {
  use super::*; // Bring everything from the outer module into scope for testing
  use std::fs;
  use std::env;
  use std::path::Path;

  fn make_temp_dir(name: &str) -> std::path::PathBuf {
    let mut temp = std::env::temp_dir();
    temp.push(name);

    std::fs::create_dir_all(&temp).unwrap();

    temp
  }

  #[test]
   fn test_load_documents_builtin() {}
}