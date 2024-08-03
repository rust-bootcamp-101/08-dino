mod build;
mod init;
mod run;

use clap::Parser;
use enum_dispatch::enum_dispatch;

pub use build::BuildOpts;
pub use init::InitOpts;
pub use run::RunOpts;

#[derive(Debug, Parser)]
#[command(name = "dino", version, author, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExecutor)]
pub enum SubCommand {
    #[command(name = "init", about = "Init dino project")]
    Init(InitOpts),

    #[command(name = "build", about = "Build dino project")]
    Build(BuildOpts),

    #[command(name = "run", about = "Run user's dino project")]
    Run(RunOpts),
}
