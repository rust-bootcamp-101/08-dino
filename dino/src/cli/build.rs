use std::env;

use clap::Parser;

use crate::{build_project, CmdExecutor};

#[derive(Debug, Parser)]
pub struct BuildOpts {}

impl CmdExecutor for BuildOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let current_dir = env::current_dir()?.display().to_string();
        let filename = build_project(&current_dir)?;
        eprintln!("Build success {}", filename);
        Ok(())
    }
}
