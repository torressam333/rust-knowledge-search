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
}
