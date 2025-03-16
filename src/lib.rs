use core::fmt;

use cli::CliParseError;
use colored::Colorize as _;

pub mod backup;
pub mod cli;
pub mod commands;
pub mod flags;
pub mod git_commands;
pub mod types;
pub mod utils;

#[derive(Debug)]
pub enum PatchyError {
    CliParseError(CliParseError),
}

impl fmt::Display for PatchyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = "error".bright_red();
        match self {
            PatchyError::CliParseError(cli_parse_error) => {
                write!(f, "{error} (Could not parse arguments): {cli_parse_error}",)
            },
        }
    }
}

impl std::error::Error for PatchyError {}

pub static CONFIG_ROOT: &str = ".patchy";
pub static CONFIG_FILE: &str = "config.toml";
pub static APP_NAME: &str = "patchy";
pub static INDENT: &str = "  ";
