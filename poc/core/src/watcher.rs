use notify::{RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{Receiver, channel};

pub fn setup() -> Result<(Receiver<String>, notify::RecommendedWatcher), anyhow::Error> {
    let (tx, rx) = channel::<String>();

    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res
                && let notify::EventKind::Modify(_) = event.kind
            {
                for path in event.paths {
                    if path.extension().and_then(|s| s.to_str()) == Some("wasm")
                        && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                    {
                        tx.send(stem.to_string()).ok();
                    }
                }
            }
        })?;

    std::fs::create_dir_all("modules").ok();
    watcher.watch(Path::new("modules"), RecursiveMode::NonRecursive)?;
    Ok((rx, watcher))
}
