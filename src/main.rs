pub mod app;
pub mod cli;
pub mod cmd;
pub mod tui;
pub mod types;
pub mod ui;

use clap::Parser;

use cli::{Cli, Command, RebuildActionArg};
use types::{ListOutput, RebuildAction};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None | Some(Command::Tui) => {
            tui::run(cli.flake).await?;
        }
        Some(Command::List) => {
            let hosts = cmd::discover_hosts(&cli.flake).await?;
            let output = ListOutput { hosts };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        Some(Command::Rebuild { host, action }) => {
            let action = match action {
                RebuildActionArg::Switch => RebuildAction::Switch,
                RebuildActionArg::Build => RebuildAction::Build,
                RebuildActionArg::DryBuild => RebuildAction::DryBuild,
            };
            let output = cmd::run_rebuild_cli(&cli.flake, &host, &action).await?;
            println!("{}", serde_json::to_string_pretty(&output)?);
            if !output.success {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
