pub mod backup;
pub mod cli;
pub mod commands;
pub mod flags;
pub mod git_commands;
pub mod types;
pub mod utils;

pub static CONFIG_ROOT: &str = ".patchy";
pub static CONFIG_FILE: &str = "config.toml";
pub static APP_NAME: &str = "patchy";
pub static INDENT: &str = "  ";
