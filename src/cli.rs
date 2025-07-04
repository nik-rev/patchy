//! Parse the command-line arguments

use clap::{
    CommandFactory as _, Parser, Subcommand, ValueEnum,
    builder::styling::{AnsiColor, Effects},
};

use crate::{
    commands,
    config::{BranchName, CommitId, PatchName, PrNumber, Remote},
};

/// A tool which makes it easy to declaratively manage personal forks by automatically merging pull requests
#[derive(Parser, Debug)]
#[command(version, styles = STYLES, long_about = None)]
pub struct Cli {
    /// Verbosity for patchy
    #[command(flatten)]
    pub verbosity: clap_verbosity_flag::Verbosity<clap_verbosity_flag::InfoLevel>,
    /// Command to invoke
    #[command(subcommand)]
    pub command: Command,
    /// Use the `gh` CLI to interact with the GitHub API
    ///
    /// This is useful if you run into github's rate limiting
    #[arg(long)]
    pub use_gh_cli: bool,
}

/// Overwrite existing patchy config file if it exists
#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum Confirm {
    /// Overwrite
    Yes,
    /// Do not overwrite
    No,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create example config file
    Init {
        /// Do not ask for confirmation when overwriting existing config file
        #[arg(short, long)]
        confirm: Option<Confirm>,
    },
    /// Invoke patchy
    Run {
        /// Do not ask for confirmation when overwriting the specified branch
        #[arg(short, long)]
        confirm: Option<Confirm>,
    },
    /// Generate a .patch file from a commit hash
    GenPatch {
        /// Transform this commit into a `.patch` file
        commit: CommitId,
        /// Choose a custom file name for the `.patch` file
        #[arg(short, long)]
        filename: Option<PatchName>,
    },
    /// Fetch pull request for a GitHub repository as a local branch
    PrFetch {
        /// Fetch PR of this number
        pr: PrNumber,
        /// The remote branch in the format `repo-owner/repo/branch`
        ///
        /// The final part (`/branch`) is optional and defaults to `main`
        ///
        /// If omitted, uses the `origin` of the current repository
        remote: Option<Remote>,
        /// Choose a custom branch name for the fetched repo
        #[arg(short, long)]
        branch: Option<BranchName>,
        /// When fetching this PR, reset to this commit
        #[arg(short = 'C', long)]
        commit: Option<CommitId>,
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
        #[arg(short = 'C', long)]
        commit: Option<CommitId>,
        /// Check out the fetched branch
        #[arg(short, long)]
        checkout: bool,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
    },
}

impl Command {
    /// Execute the command
    pub async fn execute(self, use_gh_cli: bool) -> anyhow::Result<()> {
        match self {
            Self::Init {
                confirm: overwrite_file_if_exists,
            } => commands::init(overwrite_file_if_exists)?,
            Self::Run { confirm } => commands::run(confirm, use_gh_cli).await?,
            Self::GenPatch { commit, filename } => {
                commands::gen_patch(commit, filename)?;
            }
            Self::PrFetch {
                pr,
                remote,
                branch,
                commit,
                checkout,
            } => commands::pr_fetch(pr, remote, branch, commit, checkout, use_gh_cli).await?,
            Self::BranchFetch {
                remote,
                commit,
                checkout,
            } => commands::branch_fetch(remote, commit, checkout, use_gh_cli).await?,
            Self::Completions { shell } => {
                shell.generate(&mut Cli::command(), &mut std::io::stdout());
            }
        }

        Ok(())
    }
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
    pub commit: Option<CommitId>,
}

#[cfg(test)]
mod test {
    use crate::config::{RepoName, RepoOwner};

    use super::*;

    #[test]
    fn parse_remote() {
        assert_eq!(
            "helix-editor/helix".parse::<Remote>().unwrap(),
            Remote {
                owner: RepoOwner::try_new("helix-editor").unwrap(),
                repo: RepoName::try_new("helix").unwrap(),
                branch: BranchName::try_new("main").unwrap(),
                commit: None
            }
        );
        assert_eq!(
            "helix-editor/helix/master @ 1a2b3c"
                .parse::<Remote>()
                .unwrap(),
            Remote {
                owner: RepoOwner::try_new("helix-editor").unwrap(),
                repo: RepoName::try_new("helix").unwrap(),
                branch: BranchName::try_new("master").unwrap(),
                commit: Some(CommitId::try_new("1a2b3c").unwrap())
            }
        );
        "helix-editor".parse::<Remote>().unwrap_err();
    }
}
