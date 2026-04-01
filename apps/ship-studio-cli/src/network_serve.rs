//! `ship network` — start and manage the Ship network daemon.

use anyhow::Result;

pub fn run(host: String, port: u16) -> Result<()> {
    eprintln!("ship network: starting daemon on {host}:{port}");
    eprintln!("ship network: agents connect at http://{host}:{port}/mcp");
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(ship_network::run_network(host, port))
}

pub fn status() -> Result<()> {
    ship_network::network_status()
}

pub fn stop() -> Result<()> {
    ship_network::network_stop()
}
