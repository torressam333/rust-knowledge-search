mod index;
mod ingestion;
mod search;
mod tokenizer;
mod watcher;
use clap::{Parser, Subcommand};

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
            println!("Searching for: {}", query);
        }
    }
}
