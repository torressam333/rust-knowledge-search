# Challenge 6: Folder Watcher

## Goal
Implement a folder watcher that monitors the `./notes` directory for changes and updates the index automatically when files are added, modified, or deleted.

## Tasks

### 1. Set Up the `notify` Crate
- Add the `notify` crate to your dependencies.
- Set up a watcher to monitor the `./notes` directory.

### 2. Handle File Events
- Detect file system events such as:
  - File creation
  - File modification
  - File deletion
- Log the detected events for debugging purposes.

### 3. Update the Index
- On file creation or modification:
  - Load the new or updated document.
  - Add or update the document in the index.
- On file deletion:
  - Remove the document from the index.

### 4. Design a Clean Architecture
- Ensure the folder watcher runs asynchronously.
- Use channels to communicate file events to the indexing logic.
- Avoid blocking the main thread.

### 5. Write Unit Tests
- Test the folder watcher with simulated file system events.
- Verify that the index is updated correctly for each type of event.

## Expected Outcome
- A folder watcher that monitors the `./notes` directory and updates the index in real-time.
- Robust handling of file system events and asynchronous communication.