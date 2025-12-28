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
        // Never assume 1 event = 1 path. Always iterate event.paths
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

#[cfg(test)]
mod tests {
    use crate::watcher;

    use super::*;
    use clap::builder::OsStr;
    use notify::{Event, EventKind};
    use std::path::PathBuf;
    use std::sync::mpsc::{self, Receiver};

    fn run_watcher_with_event(passed_event: notify::Event) -> Vec<super::IndexEvent> {
        // 1. Create channel
        let (tx, rx) = mpsc::channel::<super::IndexEvent>();

        // 2. Build watcher callback
        let watcher_callback = move |res: NotifyResult<Event>| {
            let event = match res {
                Ok(event) => event,
                Err(_) => return,
            };

            // 3. Translate notify OS level event kinds into domain events
            let make_index_event = match event.kind {
                notify::EventKind::Create(_) => IndexEvent::Created,
                notify::EventKind::Modify(_) => IndexEvent::Modified,
                notify::EventKind::Remove(_) => IndexEvent::Deleted,
                _ => return, // Ignore unrelated filesystem noise
            };

            // One fs event can affect multiple files so we need to loop em all
            // and send event per file
            for path in event.paths {
                let _ = tx.send(make_index_event(path));
            }
        };

        // 3. Call callback with Ok(event)
        watcher_callback(Ok(passed_event));

        // 4. Drain rx and return collected IndexEvents
        let mut rec_txs = Vec::new();

        // Keep trying to receive until the channel is empty
        while let Ok(rec) = rx.try_recv() {
            rec_txs.push(rec);
        }

        rec_txs
    }

    #[test]
    fn watcher_sends_created_event_for_txt_file() {
        // 1. Set up a channel (tx, rx)
        let (tx, rx) = mpsc::channel::<super::IndexEvent>();

        // 2. Simulate a notify::Event of kind Create with a .txt path
        let simulated_event = Event {
            kind: EventKind::Create(notify::event::CreateKind::Any),
            paths: vec![PathBuf::from("note.txt")],
            attrs: Default::default(),
        };

        // 3. simulate the fn that notify will call when fs changes
        let watcher_callback = move |res: NotifyResult<Event>| {
            let event = match res {
                Ok(event) => event,
                Err(_) => return,
            };

            // One fs event can affect multiple files so we need to loop em all
            // and send event per file
            for path in event.paths {
                let _ = tx.send(super::IndexEvent::Created(path));
            }
        };

        // 4. Call the watcher callback with the simulated event
        watcher_callback(Ok(simulated_event));

        // 5. Receive the event from rx
        let received_event = rx.recv().expect("Expected an IndexEvent");

        match received_event {
            IndexEvent::Created(path) => {
                // 7. Assert that the path matches
                assert_eq!(path, PathBuf::from("note.txt"));
            }
            _ => panic!("Expected Created event"),
        }
    }

    #[test]
    fn watcher_filters_non_txt_md_files() {
        // 1. Set up a channel
        // 2. Create watcher callback with tx
        // 3. Simulate a notify::Event of kind Create with a .jpg or .tmp file
        // 4. Send the event
        // 5. Assert that rx receives nothing
    }

    #[test]
    fn watcher_handles_multiple_paths_in_one_event() {
        // 1. Set up a channel
        // 2. Create watcher callback with tx
        // 3. Simulate a notify::Event of kind Create with multiple paths
        //    - some .txt, some .md, some ignored
        // 4. Send the event
        // 5. Receive events from rx
        // 6. Assert that only the valid .txt/.md files were sent
        // 7. Assert that the event kind is correct for each
    }

    #[test]
    fn watcher_gracefully_handles_channel_closed() {
        // 1. Set up a channel
        // 2. Drop the receiver immediately
        // 3. Call the watcher callback with a Create event
        // 4. Assert that the callback does not panic
        // 5. Assert that an error message is printed (optional)
    }

    #[test]
    fn watcher_translates_modify_and_delete_events() {
        // 1. Set up a channel
        // 2. Create watcher callback with tx
        // 3. Simulate EventKind::Modify and EventKind::Remove with valid files
        // 4. Send the events
        // 5. Assert that rx receives IndexEvent::Modified and IndexEvent::Deleted
        // 6. Assert the paths match
    }
}
