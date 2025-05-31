//! Parse the command-line arguments

use std::{path::PathBuf, str::FromStr};

use clap::{
    Parser, Subcommand,
    builder::styling::{AnsiColor, Effects},
};
use tap::Pipe as _;

use crate::git_commands::Commit;

/// Patchy automatically
#[derive(Parser)]
#[command(styles = STYLES)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create example config file
    Init,
    /// Invoke patchy
    Run {
        /// Do not prompt when overwriting local-branch specified in the config
        #[arg(short, long)]
        yes: bool,
    },
    /// Generate a .patch file from a commit hash
    GenPatch {
        /// Transform this commit into a `.patch` file
        commit: Commit,
        /// Choose a custom file name for the `.patch` file
        #[arg(short, long)]
        filename: Option<PathBuf>,
    },
    /// Fetch pull request for a GitHub repository as a local branch
    PrFetch {
        /// Fetch PR of this number
        pr: u32,
        /// The remote branch in the format `repo-owner/repo/branch`
        ///
        /// The final part (`/branch`) is optional and defaults to `main`
        ///
        /// If omitted, uses the `origin` of the current repository
        remote: Option<Remote>,
        /// Choose a custom branch name for the fetched repo
        #[arg(short, long)]
        branch: Option<String>,
        /// When fetching this PR, reset to this commit
        #[arg(short('C'), long)]
        commit: Option<Commit>,
        /// Check out the first fetched pull request
        #[arg(short, long)]
        checkout: bool,
    },
    /// Fetch branch for a GitHub repository as a local branch
    BranchFetch {
        /// The remote branch in the format `repo-owner/repo/branch`
        ///
        /// The final part (`/branch`) is optional and defaults to `main`
        remote: Remote,
        /// When fetching this branch, reset to this commit
        #[arg(short('C'), long)]
        commit: Option<Commit>,
        /// Check out the fetched branch
        #[arg(short, long)]
        checkout: bool,
    },
}

/// Styles for the CLI
const STYLES: clap::builder::Styles = clap::builder::Styles::styled()
    .header(AnsiColor::BrightGreen.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::BrightGreen.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::BrightCyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::BrightCyan.on_default())
    .error(AnsiColor::BrightRed.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::BrightCyan.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::BrightYellow.on_default().effects(Effects::BOLD));

/// Represents a single branch
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Branch {
    /// Name of the GitHub owner of the repository
    pub repo_owner: String,
    /// Name of the repository this branch belongs to
    pub repo_name: String,
    /// Name of this branch in the remote
    pub name: String,
    /// When fetching this PR, reset to this commit
    pub commit: Option<Commit>,
}

/// Example: `helix-editor/helix/master`
#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Default)]
pub struct Remote {
    /// Example: `helix-editor`
    pub owner: String,
    /// Example: `helix`
    pub repo: String,
    /// Example: `master`
    pub branch: String,
}

impl Remote {
    const DEFAULT_BRANCH: &str = "main";
}

impl FromStr for Remote {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_once('/')
            .ok_or_else(|| "Expected format: `owner/repo`".to_string())?
            .pipe(|(owner, rest)| {
                rest.split_once('/')
                    .unwrap_or((rest, Self::DEFAULT_BRANCH))
                    .pipe(|(repo, branch)| Self {
                        owner: owner.to_string(),
                        repo: repo.to_string(),
                        branch: branch.to_string(),
                    })
            })
            .pipe(Ok)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_remote() {
        assert_eq!(
            "helix-editor/helix".parse::<Remote>().unwrap(),
            Remote {
                owner: "helix-editor".to_string(),
                repo: "helix".to_string(),
                branch: "main".to_string()
            }
        );
        assert_eq!(
            "helix-editor/helix/master".parse::<Remote>().unwrap(),
            Remote {
                owner: "helix-editor".to_string(),
                repo: "helix".to_string(),
                branch: "master".to_string()
            }
        );
        "helix-editor".parse::<Remote>().unwrap_err();
    }
}
