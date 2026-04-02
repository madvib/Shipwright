//! `ship network` — start and manage the shipd daemon.

use anyhow::Result;

pub fn run(host: String, port: u16) -> Result<()> {
    eprintln!("shipd: starting daemon on {host}:{port}");
    eprintln!("shipd: agents connect at http://{host}:{port}/mcp");
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(shipd::run_network(host, port))
}

pub fn status() -> Result<()> {
    shipd::network_status()
}

pub fn stop() -> Result<()> {
    shipd::network_stop()
}
