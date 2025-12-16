use crate::ingestion::Document;
use crate::tokenizer::tokenize;
use std::collections::HashMap;
use uuid::Uuid;

pub struct Index {
    postings: HashMap<String, Vec<Uuid>>,
}

impl Index {
    pub fn new() -> Self {
        Index {
            postings: HashMap::new(),
        }
    }

    pub fn add_document(&mut self, doc: &Document) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingestion::Document;
    use std::path::PathBuf;
    use std::time::SystemTime;

    #[test]
    fn new_index_is_empty() {
        let index = Index::new();
        assert!(index.postings.is_empty());
    }

    fn add_single_document_indexes_tokens() {
        let mut index = Index::new();

        let doc = Document {
            id: Uuid::new_v4(),
            path: PathBuf::from("note.txt"),
            content: "Hello world".to_string(),
            modified: None,
        };

        index.add_document(&doc);

        assert!(index.postings.contains_key("hello"));
        assert!(index.postings.contains_key("world"));

        assert_eq!(index.postings["hello"], vec![doc.id]);
        assert_eq!(index.postings["world"], vec![doc.id]);
    }
}
