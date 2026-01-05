use notify::{Event, RecursiveMode, Result as NotifyResult, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

#[derive(Debug)]
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

        // Map OS level events to one of my custom IndexEvent
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
    use super::*;
    use notify::{Event, EventKind};
    use std::path::PathBuf;
    use std::sync::mpsc::{self};

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
                // Filter for only txt or md
                if !matches!(
                    path.extension().and_then(|e| e.to_str()),
                    Some("txt" | "md")
                ) {
                    continue;
                }

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
        // 1. Simulate a notify::Event of kind Create with a .txt path
        let simulated_event = Event {
            kind: EventKind::Create(notify::event::CreateKind::Any),
            paths: vec![
                PathBuf::from("note.txt"),
                PathBuf::from("should_be_ignored.jpg"),
            ],
            attrs: Default::default(),
        };

        // 2. Call the helper to run the watcher callback
        let index_events = run_watcher_with_event(simulated_event);

        // 3. Assert that 1 events were sent - 1 per valid ext
        assert_eq!(index_events.len(), 1);

        // Match on multiple files since 1 event can change multiple
        let paths: Vec<_> = index_events
            .iter()
            .map(|e| match e {
                IndexEvent::Created(p) => p.clone(),
                _ => panic!("Expected Created event"),
            })
            .collect();

        assert!(paths.contains(&PathBuf::from("note.txt")));
    }

    #[test]
    fn watcher_filters_non_txt_md_files() {
        // 1. Simulate a notify::Event of kind Create with a .txt path
        let simulated_event = Event {
            kind: EventKind::Create(notify::event::CreateKind::Any),
            paths: vec![
                PathBuf::from("notes.md"),
                PathBuf::from("should_be_ignored.jpg"),
            ],
            attrs: Default::default(),
        };

        // 2. Call the helper to run the watcher callback
        let index_events = run_watcher_with_event(simulated_event);

        // 3. Assert that 1 events was sent (only for md ext)
        assert_eq!(index_events.len(), 1);
    }

    #[test]
    fn watcher_gracefully_handles_channel_closed() {
        // 1. Set up a channel
        let (tx, rx) = mpsc::channel::<IndexEvent>();

        // 2. Drop the receiver immediately
        drop(rx);

        // 3. Call the watcher callback with a Create event
        let watcher_callback = move |res: NotifyResult<Event>| {
            let event = match res {
                Ok(event) => event,
                Err(_) => return,
            };

            let make_index_event = match event.kind {
                EventKind::Create(_) => IndexEvent::Created,
                _ => return,
            };

            for path in event.paths {
                // This send will FAIL because rx was dropped
                if tx.send(make_index_event(path)).is_err() {
                    // Graceful exit -> what will be asserted for
                    return;
                }
            }
        };

        // 4. Simulate sys level event
        // 4. Simulate a filesystem event
        let simulated_event = Event {
            kind: EventKind::Create(notify::event::CreateKind::Any),
            paths: vec![PathBuf::from("note.txt")],
            attrs: Default::default(),
        };

        // 5. Assert: calling the callback does NOT panic (no assertion needed)
        watcher_callback(Ok(simulated_event));
    }

    #[test]
    fn watcher_translates_modify_and_delete_events() {
        // --- MODIFY ---
        let modify_event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Any),
            paths: vec![PathBuf::from("note.txt")],
            attrs: Default::default(),
        };

        let modify_results = run_watcher_with_event(modify_event);

        assert_eq!(modify_results.len(), 1);

        match &modify_results[0] {
            IndexEvent::Modified(path) => {
                assert_eq!(path, &PathBuf::from("note.txt"));
            }
            _ => panic!("Expected Modified event"),
        }

        // --- DELETE ---
        let delete_event = Event {
            kind: EventKind::Remove(notify::event::RemoveKind::Any),
            paths: vec![PathBuf::from("old_note.md")],
            attrs: Default::default(),
        };

        let delete_results = run_watcher_with_event(delete_event);

        assert_eq!(delete_results.len(), 1);

        match &delete_results[0] {
            IndexEvent::Deleted(path) => {
                assert_eq!(path, &PathBuf::from("old_note.md"));
            }
            _ => panic!("Expected Deleted event"),
        }
    }
}
