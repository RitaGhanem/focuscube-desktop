use focuscube::WorkEvent;
use focuscube::platform::MacOSMonitor;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::sync::mpsc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("FocusCube Daemon starting...");
    
    let mut monitor = MacOSMonitor::new()?; // ← ADDED mut HERE
    
    let (file_tx, mut file_rx) = mpsc::unbounded_channel::<WorkEvent>();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if event.kind.is_modify() {
                    for path in event.paths {
                        if let Some(ext) = path.extension() {
                            if matches!(ext.to_str(), Some("rs" | "js" | "py" | "kicad_pcb")) {
                                file_tx.send(WorkEvent::FileSaved { path }).ok();
                            }
                        }
                    }
                }
            }
        },
        Config::default(),
    )?;
    
    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;
    info!("Watching current directory...");
    
    info!("Listening for events (switch apps or save files to test)...");
    
    loop {
        tokio::select! {
            Some(event) = monitor.receiver.recv() => { // ← Now works with mut monitor
                info!("📱 {}", event.to_json());
            }
            Some(event) = file_rx.recv() => {
                info!("💾 {}", event.to_json());
            }
        }
    }
}