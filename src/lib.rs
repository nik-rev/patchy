#![cfg_attr(doc, doc = include_str!("../README.md"))]
use std::env;
use std::sync::LazyLock;

mod backup;
mod cli;
mod commands;
mod commit;
mod git;
pub mod interact;
mod types;
mod utils;

static CONFIG_ROOT: LazyLock<String> =
    LazyLock::new(|| env::var("PATCHY_CONFIG_ROOT").unwrap_or_else(|_| ".patchy".into()));
const CONFIG_FILE: &str = "config.toml";
const APP_NAME: &str = "patchy";

pub use cli::Cli;
