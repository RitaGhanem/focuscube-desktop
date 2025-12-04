use crate::{EventSource, WorkEvent};
use anyhow::Result;
use tokio::sync::mpsc::UnboundedReceiver;
use std::time::Duration;
use tracing::{info, warn};
use sysinfo::{ProcessExt, System, SystemExt};

pub struct MacOSMonitor {
    pub receiver: UnboundedReceiver<WorkEvent>,
}

impl MacOSMonitor {
    pub fn new() -> Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        info!("🚀 Starting app poller using sysinfo (reliable)...");
        
        tokio::spawn(async move {
            let mut last_pid = 0u32;
            let mut sys = System::new();
            
            loop {
                sys.refresh_processes();
                
                match Self::get_active_app(&sys) {
                    Some((name, pid)) => {
                        if pid != last_pid {
                            info!("✅ APP SWITCH: {} -> {} (PID: {})", last_pid, name, pid);
                            last_pid = pid;
                            let event = WorkEvent::AppFocused { name, pid };
                            let _ = tx.send(event);
                        }
                    }
                    None => {
                        warn!("❌ No active app found");
                    }
                }
                
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
        
        Ok(Self { receiver: rx })
    }
    
    fn get_active_app(sys: &System) -> Option<(String, u32)> {
        use std::process::Command;
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to get name of first process whose frontmost is true")
            .output()
            .ok()?;
        
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if name.is_empty() {
            return None;
        }
        
        for (pid, process) in sys.processes() {
            if process.name() == name {
                return Some((name, usize::from(*pid) as u32));
            }
        }
        
        None
    }
}

#[async_trait::async_trait]
impl EventSource for MacOSMonitor {
    async fn next_event(&mut self) -> Result<WorkEvent> {
        self.receiver.recv().await.ok_or_else(|| anyhow::anyhow!("Channel closed"))
    }
}