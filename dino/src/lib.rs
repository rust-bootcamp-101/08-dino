mod cli;
mod engine;
mod utils;

pub use cli::*;
use enum_dispatch::enum_dispatch;
pub(crate) use utils::*;

pub const BUILD_DIR: &str = ".build";

#[allow(async_fn_in_trait)]
#[enum_dispatch]
pub trait CmdExecutor {
    async fn execute(self) -> anyhow::Result<()>;
}
