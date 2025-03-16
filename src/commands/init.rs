use std::fs::{self, File};
use std::io::Write as _;

use colored::Colorize as _;

use crate::git_commands::GIT_ROOT;
use crate::{CONFIG_FILE, CONFIG_ROOT, INDENT, confirm_prompt, success};

pub fn init() -> anyhow::Result<()> {
    let example_config = include_bytes!("../../example-config.toml");

    let config_path = GIT_ROOT.join(CONFIG_ROOT);

    let config_file_path = config_path.join(CONFIG_FILE);

    if config_file_path.exists()
        && !confirm_prompt!(
            "File {} already exists. Overwrite it?",
            config_file_path.to_string_lossy().bright_blue(),
        )
    {
        anyhow::bail!("Did not overwrite {config_file_path:?}");
    }

    let _ = fs::create_dir_all(config_path);

    let mut file = File::create(&config_file_path)?;

    file.write_all(example_config)?;

    success!("Created config file {config_file_path:?}");

    Ok(())
}
