use crate::ingestion::Document;
use crate::tokenizer::tokenize;
use std::collections::HashMap;
use uuid::Uuid;

pub struct Index {
    postings: HashMap<String, Vec<Uuid>>,
}

impl Index {
    pub fn add_document(&mut self, doc: &Document) {
        let tokenized_content = tokenize(&doc.content);
    }
}
