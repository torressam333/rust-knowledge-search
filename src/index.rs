use crate::ingestion::Document;
use crate::tokenizer::tokenize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Index {
    postings: HashMap<String, HashSet<Uuid>>,
    documents: HashMap<Uuid, Document>,
    path_to_id: HashMap<PathBuf, Uuid>,
    doc_tokens: HashMap<Uuid, HashSet<String>>,
}

impl Index {
    pub fn new() -> Self {
        Index {
            postings: HashMap::new(),
            documents: HashMap::new(),
            path_to_id: HashMap::new(),
            doc_tokens: HashMap::new(),
        }
    }

    pub fn add_document(&mut self, doc: Document) {
        // 1. Tokenize & dedupe
        let tokens = tokenize(&doc.content);
        let unique_tokens: HashSet<String> = tokens.into_iter().collect();

        // 2. Store tokens per document
        self.doc_tokens.insert(doc.id, unique_tokens.clone());

        // 3. Update inverted index
        for token in unique_tokens {
            self.postings
                .entry(token)
                .or_insert_with(HashSet::new)
                .insert(doc.id);
        }

        // 4. Store document & path mapping
        self.documents.insert(doc.id, doc.clone());
        self.path_to_id.insert(doc.path.clone(), doc.id);
    }

    pub fn remove_document(&mut self, doc_id: Uuid) -> () {
        if let Some(doc) = self.documents.get(&doc_id) {
            let path = doc.path.clone();
            self.path_to_id.remove(&path);
        }

        let tokens = match self.doc_tokens.get(&doc_id) {
            Some(tokens) => tokens.clone(),
            None => return,
        };

        for token in tokens {
            // remove doc_id from postings[token]
            if let Some(doc_ids) = self.postings.get_mut(&token) {
                doc_ids.remove(&doc_id);

                if doc_ids.is_empty() {
                    self.postings.remove(&token);
                }
            }
        }

        self.doc_tokens.remove(&doc_id);
        self.documents.remove(&doc_id);
    }

    pub fn search_query(&self, query: &str) -> Vec<Uuid> {
        // 1. Tokenize the query
        let tokens = tokenize(query);

        // 2. Create empty SET of doc ids
        let mut doc_ids = HashSet::new();

        // 3. Loop over tokens - if tokens exist in postings, add all doc ids to set
        for token in tokens {
            if let Some(ids) = self.postings.get(&token) {
                for uuid in ids {
                    // Deref here otherwise it will try to insert &uuid
                    // but we want doc ids to contain/return using owned Uuid
                    doc_ids.insert(*uuid);
                }
            }
        }

        // 4. conovert and return SET as a Vec<Uuid> like the sig expects
        doc_ids.into_iter().collect()
    }

    pub fn remove_document_by_path(&mut self, path: &PathBuf) {
        if let Some(doc_id) = self.path_to_id.get(path).copied() {
            self.remove_document(doc_id);
        }
    }

    pub fn upsert_document(&mut self, doc: Document) {
        // If a document already exists for this path, remove it first
        if let Some(existing_id) = self.path_to_id.get(&doc.path).copied() {
            self.remove_document(existing_id);
        }

        self.add_document(doc);
    }

    pub fn save_to_disk<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        // Walk entire index and convert to json
        let json = serde_json::to_string_pretty(self).expect("Index shouold serialize");

        // write file and handle Result
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_disk<P: AsRef<Path>>(&self, path: P) -> io::Result<Self> {}
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

        // Extract what we need before move so we can still assert
        let doc_id = doc.id;

        index.add_document(doc);

        assert!(index.postings["hello"].contains(&doc_id));
        assert!(index.postings["world"].contains(&doc_id));

        assert_eq!(index.postings["hello"].len(), 1);
        assert_eq!(index.postings["world"].len(), 1);
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

        // Grab ids before moving doc ownership
        let doc_id = doc.id;
        let doc_id_2 = doc2.id;

        index.add_document(doc);
        index.add_document(doc2);

        assert!(index.postings["hello"].contains(&doc_id));
        assert!(index.postings["friend"].contains(&doc_id_2));

        assert_eq!(index.postings["hello"].len(), 2);
        assert_eq!(index.postings["world"].len(), 2);
    }

    #[test]
    fn search_empty_query_returns_empty_vec() {
        let index = Index::new();
        let query = "";

        let search_results = index.search_query(&query);
        let empty_vec: Vec<Uuid> = Vec::new();

        assert_eq!(search_results, empty_vec);
    }

    #[test]
    fn search_single_token_returns_matching_doc() {
        let query = "I believe";
        let mut index = Index::new();

        let doc = Document {
            id: Uuid::new_v4(),
            path: PathBuf::from("note.txt"),
            content: "I believe that we will win because we are champtions at hear".to_string(),
            modified: None,
        };

        let doc_id = doc.id;
        index.add_document(doc);

        let all_docs_ids = index.search_query(query);

        assert_eq!(all_docs_ids.len(), 1);
        assert!(all_docs_ids.contains(&doc_id));
    }

    #[test]
    fn search_multiple_tokens_returns_union_of_docs() {
        let query = "believe victory";
        let mut index = Index::new();

        let doc1 = Document {
            id: Uuid::new_v4(),
            path: PathBuf::from("note1.txt"),
            content: "I believe in hard work".to_string(),
            modified: None,
        };

        let doc2 = Document {
            id: Uuid::new_v4(),
            path: PathBuf::from("note2.txt"),
            content: "Victory comes to the prepared".to_string(),
            modified: None,
        };

        let doc_id = doc1.id;
        let doc_id_2 = doc2.id;

        index.add_document(doc1);
        index.add_document(doc2);

        let results = index.search_query(query);

        assert_eq!(results.len(), 2);
        assert!(results.contains(&doc_id));
        assert!(results.contains(&doc_id_2));
    }

    #[test]
    fn search_unknown_token_returns_empty_vec() {
        let query = "I will";
        let mut index = Index::new();

        let doc = Document {
            id: Uuid::new_v4(),
            path: PathBuf::from("note.txt"),
            content: "Sometimes you just want a chicken sandwich, lol".to_string(),
            modified: None,
        };

        index.add_document(doc);

        let all_docs_ids = index.search_query(query);

        assert_eq!(all_docs_ids.len(), 0);
    }

    // Even though multiple tokens match (and one appears multiple times),
    // the document ID must appear exactly once in results.
    #[test]
    fn search_does_not_duplicate_document_ids() {
        let query = "Good morning";
        let mut index = Index::new();

        let doc = Document {
            id: Uuid::new_v4(),
            path: PathBuf::from("note.txt"),
            content: "I just want to say good morning, friends! So, good morning!".to_string(),
            modified: None,
        };

        let doc_id = doc.id;

        index.add_document(doc);

        let all_docs_ids = index.search_query(query);

        assert_eq!(all_docs_ids.len(), 1);
        assert!(all_docs_ids.contains(&doc_id));
    }

    /** Tests for the remove_document fn */
    #[test]
    fn remove_document_removes_doc_from_postings() {
        let mut index = Index::new();

        let doc = Document {
            id: Uuid::new_v4(),
            path: PathBuf::from("note.txt"),
            content: "I believe that we will win".to_string(),
            modified: None,
        };

        let doc2 = Document {
            id: Uuid::new_v4(),
            path: PathBuf::from("note.txt"),
            content: "I believe!".to_string(),
            modified: None,
        };

        let doc1_id = doc.id;
        let doc2_id = doc2.id;

        index.add_document(doc);
        index.add_document(doc2);

        index.remove_document(doc1_id);

        // Shared token still exists but only contains doc2
        let postings_for_believe = index.postings.get("believe").unwrap();
        assert!(!postings_for_believe.contains(&doc1_id));
        assert!(postings_for_believe.contains(&doc2_id));

        // Tokens unique to removed document are gone
        assert!(!index.postings.contains_key("that"));
        assert!(!index.postings.contains_key("we"));
        assert!(!index.postings.contains_key("will"));
        assert!(!index.postings.contains_key("win"));

        // Check documents map
        assert!(index.documents.get(&doc1_id).is_none());
        assert!(index.documents.get(&doc2_id).is_some());
    }

    #[test]
    fn remove_document_removes_orphaned_tokens() {
        // 1. Create a new index
        let mut index = Index::new();

        // 2. Create a document with unique tokens
        let doc = Document {
            id: Uuid::new_v4(),
            path: PathBuf::from("note.txt"),
            content: "Some unique tokens here".to_string(),
            modified: None,
        };
        let doc_id = doc.id;

        // 3. Add the document to the index
        index.add_document(doc);

        // 4. Capture all tokens for this document BEFORE removal
        let tokens_of_doc: Vec<String> = index
            .doc_tokens
            .get(&doc_id)
            .unwrap()
            .iter()
            .cloned()
            .collect();

        // Assert all tokens exist before removal
        for token in &tokens_of_doc {
            assert!(index.postings.contains_key(token));
        }

        // 5. Remove the document
        index.remove_document(doc_id);

        // 6. Assert all tokens are no longer present
        for token in tokens_of_doc {
            assert!(!index.postings.contains_key(&token));
        }

        // 7. Assert the document itself is gone
        assert!(index.documents.get(&doc_id).is_none());
        assert!(index.doc_tokens.get(&doc_id).is_none());
    }

    #[test]
    fn remove_document_cleans_up_internal_maps() {
        // 1. Create a new index
        let mut index = Index::new();

        // 2. Create a document with a path
        let doc = Document {
            id: Uuid::new_v4(),
            path: PathBuf::from("note.txt"),
            content: "Some unique tokens here".to_string(),
            modified: None,
        };

        // 3. Capture doc_id and path before moving the document
        let doc_id = doc.id;

        // 4. Add the document to the index
        index.add_document(doc);

        // 5. Assert document exists in:
        //    - documents
        //    - doc_tokens
        //    - path_to_id
        let path_buf = PathBuf::from("note.txt");
        assert!(index.documents.contains_key(&doc_id));
        assert!(index.doc_tokens.contains_key(&doc_id));
        assert_eq!(index.path_to_id.get(&path_buf), Some(&doc_id));

        // 6. Remove the document
        index.remove_document(doc_id);

        // 7. Assert document no longer exists in:
        //    - documents
        //    - doc_tokens
        //    - path_to_id
        assert!(!index.documents.contains_key(&doc_id));
        assert!(!index.doc_tokens.contains_key(&doc_id));
        assert!(index.path_to_id.get(&path_buf).is_none());
    }

    #[test]
    fn upsert_replaces_existing_document_for_same_path() {
        let mut index = Index::new();
        let path = PathBuf::from("note.txt");

        let doc1 = Document {
            id: Uuid::new_v4(),
            path: path.clone(),
            content: "hello world".to_string(),
            modified: None,
        };

        let doc2 = Document {
            id: Uuid::new_v4(),
            path: path.clone(),
            content: "goodbye world".to_string(),
            modified: None,
        };

        index.upsert_document(doc1);
        index.upsert_document(doc2);

        let results = index.search_query("goodbye");
        assert_eq!(results.len(), 1);
    }
}
