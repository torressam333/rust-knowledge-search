# Challenge 4: Build the Inverted Index

## Goal
Design and implement an `Index` that maps tokens to posting lists of document IDs. Add documents to the index and implement a basic search query function to retrieve matching document IDs.

## Tasks

### 1. Design the `Index` Struct
- Create a struct `Index` with the following field:
  - `postings: HashMap<String, Vec<Uuid>>` — Maps tokens to a list of document IDs.
- Add a `new()` method to initialize an empty `Index`.

### 2. Implement `add_document`
- Signature: `pub fn add_document(&mut self, doc: &Document)`
- Steps:
  1. Tokenize the document's content using the `tokenize` function.
  2. Deduplicate tokens for the document (use a `HashSet<String>` to avoid adding the same document ID multiple times for the same token).
  3. For each token:
     - Use `HashMap::entry` to get or create the posting list for the token.
     - Push the document ID (`doc.id`) into the posting list if it’s not already present.

### 3. Implement `search_query`
- Signature: `pub fn search_query(&self, query: &str) -> Vec<Uuid>`
- Steps:
  1. Tokenize the query string.
  2. For each token, retrieve the posting list from the `HashMap` (if it exists).
  3. Combine the posting lists for all tokens (union of document IDs).
  4. Return the resulting list of document IDs.

### 4. Write Unit Tests
- Test `add_document`:
  - Add a document and verify that its tokens are correctly indexed.
  - Add multiple documents and ensure the index is updated correctly.
- Test `search_query`:
  - Query for a single token and verify the correct document IDs are returned.
  - Query for multiple tokens and verify the union of document IDs is returned.
  - Test edge cases: empty query, tokens not in the index, etc.

## Expected Outcome
- A working `Index` struct that supports adding documents and searching for document IDs based on tokens.
- Comprehensive unit tests to validate the functionality of the `Index`.