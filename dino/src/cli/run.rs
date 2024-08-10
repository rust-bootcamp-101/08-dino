use std::fs;

use anyhow::Result;
use clap::Parser;
use dino_server::{start_server, ProjectConfig, SwappableAppRouter, TennetRouter};

use crate::{build_project, CmdExecutor};

#[derive(Debug, Parser)]
pub struct RunOpts {
    // port to listen
    #[arg(short, long, default_value = "3000")]
    pub port: u16,
}

impl CmdExecutor for RunOpts {
    async fn execute(self) -> Result<()> {
        let filename = build_project(".")?;
        let config = filename.replace(".mjs", ".yml");
        let code = fs::read_to_string(filename)?;
        let config = ProjectConfig::load(config)?;
        let router = SwappableAppRouter::try_new(&code, config.routes)?;
        let routers = vec![TennetRouter::new("localhost".to_string(), router.clone())];

        // watch_project(".", router).await?;

        start_server(self.port, routers).await?;

        Ok(())
    }
}

// fn watch_project(dir: &'static str, router: SwappableAppRouter) -> Result<()> {
//     let mut watcher = notify::recommended_watcher(move |res| match res {
//         Ok(event) => {
//             info!("event: {:?}", event);
//             let filename = build_project(dir).unwrap();
//             let config = filename.replace(".mjs", ".yml");
//             let code = fs::read_to_string(filename).unwrap();
//             let config = ProjectConfig::load(config).unwrap();
//             router.swap(code, config.routes).unwrap();
//         }
//         Err(e) => println!("watch error: {:?}", e),
//     })?;

//     watcher.watch(Path::new(dir), RecursiveMode::Recursive)?;
//     Ok(())
// }
