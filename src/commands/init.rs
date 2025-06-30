//! `init` subcommand

use std::fs::{self, File};
use std::io::Write as _;

use anyhow::bail;
use colored::Colorize as _;

use crate::{config, confirm_prompt};

/// Initialize the Patchy config file
pub fn init() -> anyhow::Result<()> {
    if config::FILE_PATH.exists()
        && !confirm_prompt!(
            "File {} already exists. Overwrite it?",
            config::FILE_PATH.to_string_lossy().bright_blue(),
        )
    {
        bail!("Did not overwrite {}", config::FILE_PATH.display());
    }

    fs::create_dir_all(&*config::PATH)?;

    let mut file = File::create(&*config::FILE_PATH)?;

    file.write_all(include_bytes!("../../example-config.toml"))?;

    log::info!("Created config file {}", config::FILE_PATH.display());

    Ok(())
}
