use core::fmt;

use cli::CliParseError;
use colored::Colorize as _;

pub mod backup;
pub mod cli;
pub mod commands;
pub mod git_commands;
pub mod types;
pub mod utils;

/// Represents errors that can be thrown by this App
#[derive(Debug)]
pub enum PatchyError {
    CliParseError(CliParseError),
}

impl fmt::Display for PatchyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = "error".bright_red().bold();
        // TODO: other kinds of errors, instead of Box<dyn Error> from main_impl
        match self {
            PatchyError::CliParseError(cli_parse_error) => {
                write!(f, "{error}: {cli_parse_error}",)
            },
        }
    }
}

impl std::error::Error for PatchyError {}

pub const CONFIG_ROOT: &str = ".patchy";
pub const CONFIG_FILE: &str = "config.toml";
pub const APP_NAME: &str = "patchy";
