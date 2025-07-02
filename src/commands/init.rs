//! `init` subcommand

use std::fs::{self, File};
use std::io::Write as _;

use anyhow::bail;
use colored::Colorize as _;

use crate::cli::Confirm;
use crate::{config, confirm_prompt};

/// Initialize the Patchy config file
pub fn init(overwrite: Option<Confirm>) -> anyhow::Result<()> {
    if config::FILE_PATH.exists() {
        let overwrite_if_exists = match overwrite {
            Some(Confirm::Yes) => true,
            Some(Confirm::No) => false,
            None => confirm_prompt!(
                "File {} already exists. Overwrite it?",
                config::FILE_PATH.to_string_lossy().bright_blue(),
            ),
        };

        if !overwrite_if_exists {
            bail!("Did not overwrite {}", config::FILE_PATH.display());
        }
    }

    fs::create_dir_all(&*config::PATH)?;

    let mut file = File::create(&*config::FILE_PATH)?;

    file.write_all(include_bytes!("../../example-config.toml"))?;

    log::info!("Created config file {}", config::FILE_PATH.display());

    Ok(())
}
