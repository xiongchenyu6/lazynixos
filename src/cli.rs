use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "A fast, modern TUI and CLI for managing NixOS flake configurations"
)]
pub struct Cli {
    /// Path to the Nix flake directory
    #[arg(long, env = "LAZYNIXOS_FLAKE", default_value = ".", global = true)]
    pub flake: PathBuf,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Launch the interactive TUI (default when no subcommand is given)
    Tui,

    /// List all nixosConfigurations discovered in the flake
    List,

    /// Run nixos-rebuild for a specific host
    Rebuild {
        /// The target host name (must match a nixosConfigurations entry)
        host: String,

        /// The rebuild action to perform
        #[arg(value_enum, default_value_t = RebuildActionArg::Switch)]
        action: RebuildActionArg,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum RebuildActionArg {
    Switch,
    Build,
    DryBuild,
}
