#![cfg_attr(doc, doc = include_str!("../docs/README.md"))]
use std::env;
use std::sync::LazyLock;

pub mod backup;
pub mod cli;
pub mod commands;
mod commit;
pub mod git_commands;
mod interact;
pub mod types;
pub mod utils;

pub static CONFIG_ROOT: LazyLock<String> =
    LazyLock::new(|| env::var("PATCHY_CONFIG_ROOT").unwrap_or_else(|_| ".patchy".into()));
pub const CONFIG_FILE: &str = "config.toml";
pub const APP_NAME: &str = "patchy";
