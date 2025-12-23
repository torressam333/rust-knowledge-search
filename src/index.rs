use crate::ingestion::Document;
use crate::tokenizer::tokenize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use uuid::Uuid;

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
}
