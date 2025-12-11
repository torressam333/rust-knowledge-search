use std::path::{Path, PathBuf};
use std::time::SystemTime;
use uuid::Uuid;

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
        let modified = entry.metadata().and_then(|m| m.modified()).ok();

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
    use std::env;
    use std::fs;
    use std::path::Path;

    fn make_temp_dir(name: &str) -> std::path::PathBuf {
        let mut temp = std::env::temp_dir();
        temp.push(name);

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
}
