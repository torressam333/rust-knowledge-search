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
    sync::{Arc, Mutex, mpsc::Receiver},
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

    // create shared index - shared across threads...neat :D
    let shared_index = Arc::new(Mutex::new(index));

    // DOnt care about message just the event
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel::<()>();

    // start watcher
    let _watcher_channel = create_watcher_channel(Arc::clone(&shared_index), shutdown_rx);

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

fn create_watcher_channel(shared_index: Arc<Mutex<Index>>, shutdown_rx: Receiver<()>) {
    let (tx, rx) = std::sync::mpsc::channel::<IndexEvent>();

    let index_clone = Arc::clone(&shared_index);

    // rx gets moved to be owned by the thread...fyi
    std::thread::spawn(move || {
        if let Err(e) = watcher::watch_notes(tx) {
            eprintln!("Watcher error: {:?}", e);
        }

        loop {
            // ----------------------------------------
            // Check for shutdown signal before doing anything
            // ----------------------------------------
            match shutdown_rx.try_recv() {
                Ok(_) => {
                    println!("Shutdown signal received, exiting watcher thread.");
                    return; // exit the thread cleanly
                }
                Err(_) => {
                    // No signal, continue processing
                }
            }

            // ----------------------------------------
            // Wait for a filesystem event
            // ----------------------------------------
            let event = match rx.recv() {
                Ok(ev) => ev,
                Err(_) => {
                    println!("Watcher channel closed, exiting watcher thread.");
                    break;
                }
            };

            // ----------------------------------------
            // Handle file reading outside of lock
            //    - Since Reading a file doesn't require access to shared Index
            // ----------------------------------------
            let doc_opt = match event {
                IndexEvent::Created(ref path) | IndexEvent::Modified(ref path) => {
                    match fs::read_to_string(path) {
                        Ok(contents) => Some((path.clone(), contents, SystemTime::now())),
                        Err(e) => {
                            eprintln!("Failed to read file {:?}: {:#?}", path, e);
                            None
                        }
                    }
                }
                IndexEvent::Deleted(_) => None, // deletion does not need file contents
            };

            // ----------------------------------------
            // ock the index ONLY when we need to mutate it
            // ----------------------------------------
            {
                let mut index = index_clone.lock().unwrap(); // lock begins

                match event {
                    IndexEvent::Created(_) | IndexEvent::Modified(_) => {
                        if let Some((path, contents, timestamp)) = doc_opt {
                            // Check if the document already exists
                            let doc_id = if let Some(existing_id) = index.path_to_id.get(&path) {
                                *existing_id
                            } else {
                                Uuid::new_v4()
                            };

                            // Build the Document struct
                            let doc = Document {
                                id: doc_id,
                                path,
                                content: contents,
                                modified: Some(timestamp),
                            };

                            // Insert or update the document in the index
                            index.upsert_document(doc);
                        }
                    }
                    IndexEvent::Deleted(ref path) => {
                        // Remove document by path
                        index.remove_document_by_path(path);
                    }
                }
            } // lock ends here

            // ----------------------------------------
            // Save the index to disk
            //    - We take a brief lock to get the snapshot, then release
            // ----------------------------------------
            if let Err(e) = shared_index.lock().unwrap().save_to_disk(INDEX_PATH) {
                eprintln!("Failed to persist index to disk: {:#?}", e);
            }
        }
    });
}
