mod config;
mod error;
mod object;
mod s3;
mod server;
mod store;

use error::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = config::Config::from_env()?;
    server::serve(config).await
}
