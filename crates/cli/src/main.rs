use anyhow::Result;
use clap::Parser;
use cli::{handle_cli, Cli};

fn main() -> Result<()> {
    let cli = Cli::parse();
    handle_cli(cli)?;
    Ok(())
}
