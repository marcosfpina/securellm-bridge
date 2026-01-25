use anyhow::Result;
use securellm_api_server::start_server;

#[tokio::main]
async fn main() -> Result<()> {
    start_server().await
}
