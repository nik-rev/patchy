//! `init` subcommand

use std::fs::{self, File};
use std::io::Write as _;

use anyhow::bail;
use colored::Colorize as _;

use crate::{CONFIG_FILE_PATH, CONFIG_PATH, confirm_prompt};

/// Initialize the Patchy config file
pub fn init() -> anyhow::Result<()> {
    if CONFIG_FILE_PATH.exists()
        && !confirm_prompt!(
            "File {} already exists. Overwrite it?",
            CONFIG_FILE_PATH.to_string_lossy().bright_blue(),
        )
    {
        bail!("Did not overwrite {}", CONFIG_FILE_PATH.display());
    }

    fs::create_dir_all(&*CONFIG_PATH)?;

    let mut file = File::create(&*CONFIG_FILE_PATH)?;

    file.write_all(include_bytes!("../../example-config.toml"))?;

    log::info!("Created config file {}", CONFIG_FILE_PATH.display());

    Ok(())
}
