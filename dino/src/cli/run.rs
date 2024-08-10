use std::{fs, path::Path, time::Duration};

use anyhow::Result;
use clap::Parser;
use dino_server::{start_server, ProjectConfig, SwappableAppRouter, TennetRouter};
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use tokio::sync::mpsc::channel;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tracing::{info, warn};

use crate::{build_project, CmdExecutor};

#[derive(Debug, Parser)]
pub struct RunOpts {
    // port to listen
    #[arg(short, long, default_value = "3000")]
    pub port: u16,
}

impl CmdExecutor for RunOpts {
    async fn execute(self) -> Result<()> {
        let (code, config) = get_code_and_config()?;
        let router = SwappableAppRouter::try_new(&code, config.routes)?;
        let routers = vec![TennetRouter::new("localhost".to_string(), router.clone())];

        tokio::spawn(async_watch(".", router));

        start_server(self.port, routers).await?;

        Ok(())
    }
}

fn get_code_and_config() -> Result<(String, ProjectConfig)> {
    let filename = build_project(".")?;
    let config = filename.replace(".mjs", ".yml");
    let code = fs::read_to_string(filename)?;
    let config = ProjectConfig::load(config)?;
    Ok((code, config))
}

const MONITOR_FS_INTERVAL: Duration = Duration::from_secs(2);

async fn async_watch(p: impl AsRef<Path>, router: SwappableAppRouter) -> Result<()> {
    let (tx, rx) = channel(1);

    // Select recommended watcher for debouncer.
    // Using a callback here, could also be a channel.
    let mut debouncer = new_debouncer(MONITOR_FS_INTERVAL, move |res: DebounceEventResult| {
        if let Err(e) = tx.blocking_send(res) {
            warn!("Failed to send debouncer event: {:?}", e);
        }
    })?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    debouncer
        .watcher()
        .watch(p.as_ref(), RecursiveMode::Recursive)?;
    let mut stream = ReceiverStream::new(rx);
    while let Some(ret) = stream.next().await {
        match ret {
            Ok(events) => {
                let mut need_swap = false;
                // config.yml change, or any ".ts" / ".js" file change
                for event in events {
                    let path = event.path;
                    let ext = path.extension().unwrap_or_default();
                    if path.ends_with("config.yml") || ext == "ts" || ext == "js" {
                        info!("File changed: {}", path.display());
                        need_swap = true;
                        break;
                    }
                }
                if need_swap {
                    let (code, config) = get_code_and_config()?;
                    router.swap(code, config.routes)?;
                }
            }
            Err(e) => {
                warn!("Error: {:?}", e);
            }
        }
    }
    Ok(())
}
