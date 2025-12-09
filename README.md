# Rust Personal Knowledge Search Engine (PKSE)

## Project Overview

The Rust Personal Knowledge Search Engine (PKSE) is a **local, offline tool** that allows you to search through your personal notes stored as `.md` or `.txt` files. It combines a **CLI tool** with an optional **HTTP server** to provide fast, searchable access to your documents.

PKSE is designed as a **learning project for Rust**, emphasizing ownership, borrowing, error handling, modular design, and idiomatic Rust practices.

---

## Features

* **Folder Watching:** Automatically watches the `./notes` directory and updates the index when files change.
* **Document Loader:** Reads and processes `.md` and `.txt` files into structured `Document` objects.
* **Tokenizer:** Normalizes and tokenizes text into words for indexing.
* **Inverted Index:** Maps words to the documents they appear in, enabling fast search.
* **CLI Search Tool:** Allows you to search your notes directly from the terminal.
* **Optional HTTP Server:** Provides a `/search` endpoint via Axum for programmatic access.
* **Rust Engineering Practices:** Modular design, proper error handling, tests, and documentation.

---

## Learning Goals

This project is not just about building a search engine, it’s about **learning Rust deeply** through practical implementation:

* Understanding ownership, borrowing, and lifetimes
* File I/O and error handling (`Result`, `Option`)
* Working with `HashMap` and other collections
* Modular project structure and separation of concerns
* Async programming and event-driven design
* Writing tests and documentation

---

## Project Structure

```
pkse/
├── src/
│   ├── main.rs      # Entry point
│   ├── index.rs     # Inverted index logic
│   ├── watcher.rs   # Folder watcher logic
│   └── search.rs    # CLI and HTTP search functions
│   └── ingestion.rs # Everything related to loading files
└── Cargo.toml       # Cargo configuration
└── README.md        # Repo readme/info file
```

---

## How to Use

### CLI Search

```bash
cargo run -- search "your query"
```

### HTTP Server (Optional)

1. Run the server

```bash
cargo run -- server
```

2. Access search endpoint

```
GET http://localhost:3000/search?q=your+query
```

---

## Getting Started

1. Clone the repository
2. Create a `notes/` folder and add `.md` or `.txt` files
3. Build and run with `cargo run`
4. Start searching with the CLI or optional HTTP server

---

## Contribution and Learning

This project is meant for **learning Rust hands-on**. You are encouraged to:

* Read Rust documentation and examples
* Experiment with the code
* Only ask AI for guidance, not full solutions
* Add tests and improve modularity

---

## License

This project is open for personal learning purposes.
