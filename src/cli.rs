use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the Nix flake directory
    #[arg(
        long,
        env = "LAZYNIXOS_FLAKE",
        default_value = "."
    )]
    pub flake: PathBuf,
}
