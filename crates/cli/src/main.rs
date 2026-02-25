use anyhow::Result;
use clap::Parser;
use cli::{Cli, handle_cli};

fn main() -> Result<()> {
    let cli = Cli::parse();
    handle_cli(cli)?;
    Ok(())
}
