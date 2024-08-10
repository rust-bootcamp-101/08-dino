use anyhow::Result;
use dino_server::{start_server, ProjectConfig, SwappableAppRouter, TennetRouter};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = include_str!("../fixtures/config-server.yml");
    let config: ProjectConfig = serde_yml::from_str(config)?;
    let code = r#"
        (function(){
            async function hello(req){
                return {
                    status:200,
                    headers:{
                        "content-type":"application/json"
                    },
                    body: JSON.stringify(req)
                };
            }
            return{hello:hello};
        })()"#;

    let routers = vec![TennetRouter::new(
        "localhost".to_string(),
        SwappableAppRouter::try_new(code, config.routes)?,
    )];
    start_server(8888, routers).await?;

    Ok(())
}
