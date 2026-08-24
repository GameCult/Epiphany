use anyhow::Result;
use epiphany_openai_runtime::{EpiphanyModelConnectorOptions, serve_model_connector};

#[tokio::main]
async fn main() -> Result<()> {
    serve_model_connector(EpiphanyModelConnectorOptions::from_env_args()?).await
}
