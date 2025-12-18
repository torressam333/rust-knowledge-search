mod index;
mod ingestion;
mod search;
mod tokenizer;
mod watcher;
use clap::{Parser, Subcommand};
use std::{path::PathBuf, time::SystemTime};
use uuid::Uuid;

use crate::tokenizer::tokenize;

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
    /// Search documents by query string
    Search {
        /// The search query
        query: String,
    },
}

fn main() {
    // Parse command-line arguments into Cli struct
    let cli = Cli::parse();

    match cli.command {
        Commands::Search { query } => {
            run_search(query);
        }
    }
}

fn run_search(query: String) {
    let tokens = tokenize(&query);

    println!("tokens from query ={:#?}", tokens);

    // Create new index
    let mut index = index::Index::new();

    // mock doc for now
    let mock_doc = ingestion::Document {
        id: Uuid::new_v4(),
        path: PathBuf::new(),
        content: "Hi There!".to_string(),
        modified: Some(SystemTime::now()),
    };

    index.add_document(&mock_doc);

    index.search_query(&query); // ->>> doesnt exist implement in index.rs
}
