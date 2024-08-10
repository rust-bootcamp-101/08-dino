use anyhow::Result;
use clap::Parser;
use dino::{CmdExecutor, Opts};

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let opts = Opts::parse();
    opts.cmd.execute().await?;
    Ok(())
}
