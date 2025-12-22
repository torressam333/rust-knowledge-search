use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;
use uuid::Uuid;

// TODO: Making fields pub for now...will add getters leter to make more robust.
#[derive(Debug, Clone)]
pub struct Document {
    pub id: Uuid,
    pub path: PathBuf,
    pub content: String,
    pub modified: Option<SystemTime>,
}

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("path is not a directory")]
    NotDirectory,
    /// Wrap any underlying I/O error. `#[from]` creates `From<std::io::Error>`,
    /// which lets `?` convert `std::io::Error` -> `IngestError::Io(...)` automatically.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn load_documents(dir: &Path) -> Result<Vec<Document>, IngestError> {
    // 1. Ensure the path is a directory
    if !dir.is_dir() {
        return Err(IngestError::NotDirectory);
    }

    // 2. Read directory entries
    // Before: std::fs::read_dir(dir).map_err(IngestError::Io)?
    // Now: the `?` will convert `std::io::Error` -> `IngestError` via `From attribute
    let entries = std::fs::read_dir(dir)?;

    let mut docs = Vec::new();

    // 3. Process each entry
    for entry_result in entries {
        let entry = entry_result?;
        let path = entry.path();

        // Skip things that aren't files (avoid indexing directories)
        // Use file_type() to avoid a second metadata syscall in some systems.
        if !entry.file_type()?.is_file() {
            continue;
        }

        // 4. Only allow .md or .txt files
        let is_text = matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("md" | "txt")
        );

        if !is_text {
            continue;
        }

        // 5. Read the file contents (propagates io::Error -> IngestError::Io)
        let content = read_to_string(&path)?;

        // 6. Get modified time
        // Ignore metadata errors and dont fail the whole load:
        let modified = entry.metadata().ok().and_then(|m| m.modified().ok());

        // 7. Build the document
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

    fn make_temp_dir(name: &str) -> std::path::PathBuf {
        let mut temp = std::env::temp_dir();
        let unique_name = format!("{}_{}", name, Uuid::new_v4());

        temp.push(unique_name);

        if temp.exists() {
            std::fs::remove_dir_all(&temp).unwrap();
        }

        std::fs::create_dir_all(&temp).unwrap();

        temp
    }

    #[test]
    fn test_load_documents_builtin() {
        let dir_path = make_temp_dir("rust_test_notes");

        // Write a txt file
        let file_path = dir_path.join("note1.txt");
        fs::write(&file_path, "hello beautiful world").unwrap();

        // Write an md file
        let file_path2 = dir_path.join("note2.md");
        fs::write(&file_path2, "Some markdown content").unwrap();

        // Write a non-text file (should be ignored)
        let file_path3 = dir_path.join("some_image.png");
        fs::write(&file_path3, "binary").unwrap();

        // Call the loader
        let docs = load_documents(&dir_path).unwrap();

        // Lets make some assertions :)
        assert_eq!(docs.len(), 2); // -> should only pick up the txt and md file if workin correctly
        assert!(
            docs.iter()
                .any(|doc| doc.content == "hello beautiful world")
        );
        assert!(
            docs.iter()
                .any(|doc| doc.content == "Some markdown content")
        );
    }

    // Test the error case (NotDirectory)
    #[test]
    fn test_not_directory_builtin() {
        let dir_path = make_temp_dir("rust_test_notes");

        // Create a file instead of directory
        let tmp_file = dir_path.join("not_a_dir.txt");
        fs::write(&tmp_file, "oops").unwrap();

        // Call loader, expect error
        let err = load_documents(&tmp_file).unwrap_err();
        match err {
            IngestError::NotDirectory => (), // expected
            _ => panic!("Expected NotDirectory"),
        }
    }
}
