use std::fs;

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
    async fn execute(self) -> anyhow::Result<()> {
        let filename = build_project(".")?;
        let config = filename.replace(".mjs", ".yml");
        let code = fs::read_to_string(filename)?;
        let config = ProjectConfig::load(config)?;
        let routers = vec![TennetRouter::new(
            "localhost".to_string(),
            SwappableAppRouter::try_new(&code, config.routes)?,
        )];
        start_server(self.port, routers).await?;

        Ok(())
    }
}
