# Challenge 7: Optional HTTP Server

## Goal
Build an optional HTTP server using the `Axum` framework to expose a `/search` endpoint for querying the index programmatically.

## Tasks

### 1. Set Up Axum
- Add the `axum` crate to your dependencies.
- Create a basic HTTP server that listens on a configurable port (e.g., `localhost:3000`).

### 2. Define the `/search` Endpoint
- Implement a `GET /search` endpoint that accepts a query string as a parameter (e.g., `?q=rust`).
- Pass the query string to the `search_query` method of the `Index`.
- Return the matching document IDs as a JSON response.

### 3. Shared State
- Use `Arc<Mutex<Index>>` or another concurrency-safe mechanism to share the `Index` between the folder watcher and the HTTP server.
- Ensure thread safety when accessing or modifying the `Index`.

### 4. Error Handling
- Handle cases where the query string is missing or invalid.
- Return appropriate HTTP status codes and error messages.

### 5. Write Unit Tests
- Test the `/search` endpoint with various query strings.
- Test edge cases, such as empty queries or no matching results.
- Test concurrent requests to ensure thread safety.

## Expected Outcome
- A functional HTTP server that exposes a `/search` endpoint.
- Properly formatted JSON responses and robust error handling.