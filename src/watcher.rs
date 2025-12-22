/**
 * Watcher exists to keep your index in sync with the filesystem
 * Somewhat of a pub sub pattern used here
 *
 * tx is transmitter
 * rx is receiver (main thread most likely)
 */
use notify::Result as NotifyResult;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub enum IndexEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

// fully explicit Result type
pub fn watch_notes(tx: Sender<IndexEvent>) -> NotifyResult<()> {
    Ok(())
}
