//! Patchy

#![cfg_attr(doc, doc = include_str!("../README.md"))]
use std::env;
use std::sync::LazyLock;

mod cli;
mod commands;
mod commit;
mod git;
mod github_api;
pub mod interact;
mod utils;

/// Root of patchy's configuration
static CONFIG_ROOT: LazyLock<String> =
    LazyLock::new(|| env::var("PATCHY_CONFIG_ROOT").unwrap_or_else(|_| ".patchy".into()));
/// Patchy's config file name
const CONFIG_FILE: &str = "config.toml";

pub use cli::Cli;
