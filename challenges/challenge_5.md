# Challenge 5: CLI Search Tool

## Goal
Build a command-line interface (CLI) tool that allows users to search the indexed documents by providing a query string.

## Tasks

### 1. Argument Parsing
- Use the `clap` crate to parse command-line arguments.
- Define a `search` subcommand that accepts a query string as an argument.

### 2. Integrate with the Index
- Load the index from the existing data.
- Pass the query string to the `search_query` method of the `Index`.
- Retrieve and display the matching document IDs.

### 3. Format Output
- Display the results in a user-friendly format:
  - Document IDs
  - Optionally, document paths or snippets of content.

### 4. Error Handling
- Handle cases where the index is empty or the query string is invalid.
- Provide meaningful error messages to the user.

### 5. Write Unit Tests
- Test the CLI argument parsing.
- Test the integration with the `Index`.
- Test edge cases, such as empty queries or no matching results.

## Expected Outcome
- A functional CLI tool that allows users to search the indexed documents.
- Properly formatted output and robust error handling.