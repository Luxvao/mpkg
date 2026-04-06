mod cli;

use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;

    mpkg_core::directory::init()?;

    cli::init()
}
