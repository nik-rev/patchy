//! Patchy

#![cfg_attr(doc, doc = include_str!("../README.md"))]
use std::env;
use std::path::PathBuf;
use std::sync::LazyLock;

mod cli;
mod commands;
mod commit;
mod config;
mod git;
mod github_api;
mod utils;

/// Relative path to root of patchy's configuration
static CONFIG_ROOT: LazyLock<String> =
    LazyLock::new(|| env::var("PATCHY_CONFIG_ROOT").unwrap_or_else(|_| ".patchy".into()));

/// Absolute path to root of patchy's configuration
static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| GIT_ROOT.join(&*CONFIG_ROOT));

/// Absolute path to patchy's config file
static CONFIG_FILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_PATH.join(CONFIG_FILE));

/// Patchy's config file name
const CONFIG_FILE: &str = "config.toml";

pub use cli::Cli;
use git::GIT_ROOT;
