mod index;
mod ingestion;
mod search;
mod tokenizer;
mod watcher;
use crate::index::Index;
use crate::ingestion::Document;
use crate::tokenizer::tokenize;
use crate::watcher::IndexEvent;
use clap::{Parser, Subcommand};
use std::{
    fs,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "rust-search")]
#[command(about = "Search indexed documents", long_about = None)]
struct Cli {
    // The command to run
    #[command(subcommand)]
    command: Commands,
}

// All supported subcommands
#[derive(Subcommand, Debug)]
enum Commands {
    Search {
        /// The search query
        query: String,
    },
}

const INDEX_PATH: &str = "index.json";

fn main() {
    let cli = Cli::parse();

    // Load index if exists else create as new
    let index = Index::load_from_disk(INDEX_PATH).unwrap_or_else(|_| Index::new());

    // create shared index
    let shared_index = Arc::new(Mutex::new(index));

    // start watcher
    let _watcher_channel = create_watcher_channel(Arc::clone(&shared_index));

    // handle CLI commands
    match cli.command {
        Commands::Search { query } => {
            run_search(query, Arc::clone(&shared_index));
        }
    }
}

fn run_search(query: String, shared_index: Arc<Mutex<Index>>) {
    let tokens = tokenize(&query);
    println!("tokens from query ={:#?}", tokens);

    // Lock index for reading
    let index = shared_index.lock().unwrap();

    let results = index.search_query(&query);
    println!("Found {} results", results.len());
}

fn create_watcher_channel(shared_index: Arc<Mutex<Index>>) {
    let (tx, rx) = std::sync::mpsc::channel::<IndexEvent>();

    let index_clone = Arc::clone(&shared_index);

    // rx gets moved to be owned by the thread...fyi
    std::thread::spawn(move || {
        if let Err(e) = watcher::watch_notes(tx) {
            eprintln!("Watcher error: {:?}", e);
        }

        // Loop to receive events and update the index
        loop {
            match rx.recv() {
                Ok(event) => {
                    let mut index = index_clone.lock().unwrap();
                    match event {
                        IndexEvent::Created(path) | IndexEvent::Modified(path) => {
                            if let Ok(contents) = fs::read_to_string(&path) {
                                // Reuse same doc id to prevent dupes or create new if not existent
                                let doc_id = if let Some(existing_id) = index.path_to_id.get(&path)
                                {
                                    *existing_id
                                } else {
                                    Uuid::new_v4()
                                };

                                let doc = Document {
                                    id: doc_id,
                                    path,
                                    content: contents,
                                    modified: Some(SystemTime::now()),
                                };

                                index.upsert_document(doc);

                                // Crash on failure b/c disk write failure is serious
                                index
                                    .save_to_disk(INDEX_PATH)
                                    .expect("Failed to persist index");
                            }
                        }
                        IndexEvent::Deleted(path) => {
                            index.remove_document_by_path(&path);

                            // Crash on failure b/c disk write failure is serious
                            index
                                .save_to_disk(INDEX_PATH)
                                .expect("Failed to persist index");
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });
}
