#[tokio::main]
async fn main() -> anyhow::Result<()> {
    mcp::run().await
}
