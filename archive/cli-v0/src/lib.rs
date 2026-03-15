#![allow(dead_code)]

use anyhow::Result;

mod app;
mod surface;

pub use app::{append_doctor_checks, handle_cli, handle_init_command};
pub use surface::*;

pub struct ShipCliApp;

impl cli_framework::CliApp for ShipCliApp {
    type Cli = Cli;

    fn metadata() -> cli_framework::CliMetadata {
        cli_framework::CliMetadata::new("ship-cli", "Ship CLI", env!("SHIP_VERSION_STRING"))
    }

    fn classify_core_command(cli: &Self::Cli) -> Option<cli_framework::CoreCommand> {
        match &cli.command {
            Some(Commands::Init { path }) => {
                Some(cli_framework::CoreCommand::Init { path: path.clone() })
            }
            Some(Commands::Doctor) => Some(cli_framework::CoreCommand::Doctor),
            Some(Commands::Version) => Some(cli_framework::CoreCommand::Version),
            _ => None,
        }
    }

    fn init(
        target: cli_framework::InitTarget,
        _context: &cli_framework::CliRunContext,
    ) -> Result<()> {
        handle_init_command(target)
    }

    fn doctor(
        report: &mut cli_framework::DoctorReport,
        _context: &cli_framework::CliRunContext,
    ) -> Result<()> {
        append_doctor_checks(report)
    }

    fn version_details() -> Vec<cli_framework::VersionDetail> {
        vec![
            cli_framework::VersionDetail::new(
                "git_hash",
                option_env!("SHIP_GIT_SHA").unwrap_or("unknown"),
            ),
            cli_framework::VersionDetail::new(
                "git_commits",
                option_env!("SHIP_GIT_COMMIT_COUNT").unwrap_or("unknown"),
            ),
            cli_framework::VersionDetail::new(
                "dirty",
                if option_env!("SHIP_GIT_DIRTY").unwrap_or("0") == "1" {
                    "yes"
                } else {
                    "no"
                },
            ),
            cli_framework::VersionDetail::new(
                "built_at",
                option_env!("SHIP_BUILD_TIMESTAMP").unwrap_or("unknown"),
            ),
        ]
    }

    fn handle(cli: Self::Cli, _context: &cli_framework::CliRunContext) -> Result<()> {
        handle_cli(cli)
    }
}

pub fn run() -> Result<()> {
    cli_framework::run::<ShipCliApp>()
}
