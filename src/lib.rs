//! Patchy

#![cfg_attr(doc, doc = include_str!("../README.md"))]

mod cli;
mod commands;
mod config;
mod git;
mod github;
mod utils;

pub use cli::Cli;
