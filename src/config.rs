//! Patchy's config

use anyhow::anyhow;
use std::fs;

use indexmap::IndexSet;
use serde::Deserialize;

use crate::{CONFIG_FILE, CONFIG_FILE_PATH, CONFIG_ROOT, commands, commit::Commit, confirm_prompt};

/// Represents the TOML config
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// Local branch where patchy will do all of its work
    pub local_branch: String,
    /// List of patches to apply
    #[serde(default)]
    pub patches: IndexSet<String>,
    /// List of pull request to apply
    #[serde(default)]
    pub pull_requests: Vec<Ref>,
    /// List of branches to apply
    #[serde(default)]
    pub branches: Vec<Ref>,
    /// Branch of the remote repository
    pub remote_branch: Ref,
    /// Remote repository where all of the `branches` and `pull_requests` are
    pub repo: String,
}

impl Config {
    /// Read the `Config`. If `yes`, will not ask for any confirmation
    pub fn read(yes: bool) -> anyhow::Result<Option<Self>> {
        let root = CONFIG_ROOT.as_str();

        let Ok(config_string) = fs::read_to_string(&*CONFIG_FILE_PATH) else {
            log::error!("Could not find configuration file at {root}/{CONFIG_FILE}");

            // We don't want to have *any* sort of prompt when using the -y flag since that
            // would be problematic in scripts
            if !yes && confirm_prompt!("Would you like us to run `patchy init` to initialize it?",)
            {
                commands::init()?;
            } else if yes {
                log::info!("You can create it with `patchy init`",);
            } else {
                // user said "no" in the prompt, so we don't do any initializing
            }

            // We don't want to read the default configuration file as config_string. Since
            // it's empty there's no reason why the user would want to run it.

            return Ok(None);
        };

        log::trace!("Using configuration file {}", CONFIG_FILE_PATH.display());

        let config = toml::from_str::<Config>(&config_string).map_err(|err| {
            anyhow!("Could not parse `{root}/{CONFIG_FILE}` configuration file:\n{err}",)
        })?;

        Ok(Some(config))
    }
}

/// Represents any git item which may be associated with a commit
#[derive(Debug)]
pub struct Ref {
    /// Git item. E.g. branch, or remote which may associate with the `commit`
    pub item: String,
    /// Commit to checkout of the `item`. If none, uses the latest commit
    pub commit: Option<Commit>,
}

impl Ref {
    /// Parses user inputs of the form `<head> @ <commit-hash>`
    pub fn new(input: &str) -> Self {
        let parts: Vec<_> = input
            // Allow users to prefix their PRs with octothorpe, e.g. `#12345` instead of
            // `12345`. This is just a QOL addition since some people may use it due to habit
            .strip_suffix('#')
            .unwrap_or(input)
            .split(" @ ")
            .collect();

        let len = parts.len();

        if len == 1 {
            // The string does not contain the <syntax>, so the user chose to use the latest
            // commit rather than a specific one
            Self {
                item: input.into(),
                commit: None,
            }
        } else {
            // They want to use a specific commit
            let head: String = parts[0..len - 1].iter().map(|s| String::from(*s)).collect();
            let commit = (parts[len - 1].to_owned()).parse::<Commit>().ok();
            Self { item: head, commit }
        }
    }
}

impl<'de> Deserialize<'de> for Ref {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Ref::new(&String::deserialize(deserializer)?))
    }
}
