use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use mpkg_core::package::{
    build_package, check_all_packages, launch_package_bin, list_all_packages, uninstall_package,
};

#[derive(Parser)]
#[command(
    version,
    long_about = "Simple packaging program with auto-update capabilities"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Builds a package")]
    Build { package: String },
    #[command(about = "Checks for updates")]
    Check,
    #[command(about = "Lists packages")]
    List,
    #[command(about = "Launch a package")]
    Launch { package: String },
    #[command(about = "Uninstall a package")]
    Uninstall { package: String },
}

pub fn init() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { package } => build_package(&package)?,
        Commands::Check => check_all_packages()?,
        Commands::List => list_all_packages()?,
        Commands::Launch { package } => {
            let _ = launch_package_bin(&package)?;
        }
        Commands::Uninstall { package } => uninstall_package(&package)?,
    }

    Ok(())
}
