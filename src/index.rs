use crate::ingestion::Document;
use crate::tokenizer::tokenize;
use std::collections::HashMap;
use std::collections::HashSet;
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
        let tokens = tokenize(&doc.content);

        // Prevent dupe doc id's per token
        let mut unique_tokens = HashSet::new();

        for token in tokens {
            unique_tokens.insert(token);
        }

        for token in unique_tokens {
            self.postings
                .entry(token)
                .or_insert_with(Vec::new)
                .push(doc.id);
        }
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

    #[test]
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

    #[test]
    fn add_two_docs() {
        let mut index = Index::new();

        let doc = Document {
            id: Uuid::new_v4(),
            path: PathBuf::from("note.txt"),
            content: "Hello world".to_string(),
            modified: None,
        };

        let doc2 = Document {
            id: Uuid::new_v4(),
            path: PathBuf::from("note.txt"),
            content: "Hello world how are you friend?".to_string(),
            modified: None,
        };

        index.add_document(&doc);
        index.add_document(&doc2);

        assert!(index.postings.contains_key("hello"));
        assert!(index.postings.contains_key("world"));
        assert_eq!(index.postings["hello"], vec![doc.id, doc2.id]);
        assert_eq!(index.postings["friend"], vec![doc2.id]);
    }
}
