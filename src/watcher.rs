use notify::{Event, RecursiveMode, Result as NotifyResult, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

pub enum IndexEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

// Listen to filesystem events and publish IndexEvents.
pub fn watch_notes(tx: Sender<IndexEvent>) -> NotifyResult<()> {
    // 1. Create a filesystem watcher with a callback
    let mut watcher = notify::recommended_watcher(move |res| {
        // 2. Handle notify-level errors defensively
        let event: Event = match res {
            Ok(event) => event,
            Err(e) => {
                eprintln!("watch error: {:?}", e);
                return;
            }
        };

        // 3. Translate notify OS level event kinds into domain events
        let make_index_event = match event.kind {
            notify::EventKind::Create(_) => IndexEvent::Created,
            notify::EventKind::Modify(_) => IndexEvent::Modified,
            notify::EventKind::Remove(_) => IndexEvent::Deleted,
            _ => return, // Ignore unrelated filesystem noise
        };

        // 4. Handle each affected path independently
        for path in event.paths {
            // 5. Filter for only files we care about (.txt / .md)
            if !matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("txt" | "md")
            ) {
                continue;
            }

            // 6. Send a domain level event to the indexer
            if tx.send(make_index_event(path.clone())).is_err() {
                // Receiver is gone then just do a graceful shutdown
                eprintln!("index receiver dropped; stopping watcher");
                return;
            }
        }
    })?;

    // 7. Start watching the ./notes directory recursively
    watcher.watch(Path::new("./notes"), RecursiveMode::Recursive)?;

    // 8. Keep the watcher alive for the lifetime of the program
    loop {
        std::thread::park();
    }
}
