mod book;
mod docs;
mod reference;
mod server;
mod tools;

use rmcp::ServiceExt;
use server::MelodiumMcp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = MelodiumMcp::new().serve(rmcp::transport::stdio()).await?;
    server.waiting().await?;
    Ok(())
}
