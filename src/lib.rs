//! Patchy

#![cfg_attr(doc, doc = include_str!("../README.md"))]

mod cli;
mod commands;
mod config;
mod git;
mod git_high_level;
mod github_api;
mod utils;

pub use cli::Cli;
