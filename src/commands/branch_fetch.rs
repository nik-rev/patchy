use colored::Colorize as _;

use super::run::parse_if_maybe_hash;
use crate::cli::branch_fetch::BranchFetch;
use crate::flags::Flag;
use crate::git_commands::{GIT, fetch_branch};
use crate::{fail, success};

pub static BRANCH_FETCH_CHECKOUT_FLAG: Flag<'static> = Flag {
    short: "-c",
    long: "--checkout",
    description: "Check out the first fetched branch",
};

pub static BRANCH_FETCH_BRANCH_NAME_FLAG: Flag<'static> = Flag {
    short: "-b=",
    long: "--branch-name=",
    description: "Choose local name for the branch belonging to the preceding pull request",
};

pub static BRANCH_FETCH_FLAGS: &[&Flag<'static>; 2] =
    &[&BRANCH_FETCH_BRANCH_NAME_FLAG, &BRANCH_FETCH_CHECKOUT_FLAG];

pub struct Item {
    /// # Examples
    ///
    /// helix-editor/helix
    pub repo: String,
    /// # Examples
    ///
    /// master
    pub branch: String,
    /// If specified, use a custom branch name instead of a generated one
    ///
    /// # Examples
    ///
    /// my-custom-branch123
    pub local_branch_name: Option<String>,
    /// If specified, do a **hard reset** to this commit when fetching the
    /// branch
    ///
    /// # Examples
    ///
    /// 6049f2035
    pub commit_hash: Option<String>,
}

impl Item {
    pub fn new(
        repo: String,
        branch: String,
        local_branch_name: Option<String>,
        commit_hash: Option<String>,
    ) -> Self {
        Self {
            repo,
            branch,
            local_branch_name,
            commit_hash,
        }
    }

    pub fn create(arg: &str) -> anyhow::Result<Self> {
        let (remote, hash) = parse_if_maybe_hash(arg, "@");

        let (repo, branch) = remote.rsplit_once('/').ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid format: {}, skipping. Valid format is: username/repo/branch. Example: \
                 helix-editor/helix/master",
                remote
            )
        })?;

        Ok(Self::new(repo.to_owned(), branch.to_owned(), None, hash))
    }

    #[must_use]
    pub fn with_branch_name(mut self, branch_name: Option<String>) -> Self {
        self.local_branch_name = branch_name;
        self
    }
}

pub async fn branch_fetch(args: BranchFetch) -> anyhow::Result<()> {
    #[expect(
        clippy::unused_enumerate_index,
        reason = "The commented code will use this. TODO"
    )]
    for (_i, branch) in args.branches.into_iter().enumerate() {
        match fetch_branch(&branch).await {
            Ok((_, info)) => {
                success!(
                    "Fetched branch {}/{}/{} available at branch {}{}",
                    branch.repo_owner,
                    branch.repo_name,
                    info.branch.upstream_branch_name,
                    info.branch.local_branch_name.bright_cyan(),
                    branch
                        .commit
                        .map(|commit_hash| format!(", at commit {}", commit_hash.bright_yellow()))
                        .unwrap_or_default()
                );

                // Attempt to cleanup after ourselves
                let _ = GIT(&["remote", "remove", &info.remote.local_remote_alias]);

                // If user uses --checkout flag, we're going to checkout the
                // first fetched branch
                // if i == 0 && args.checkout {
                //     if let Err(cant_checkout) = GIT(&["checkout",
                // &info.branch.local_branch_name]) {
                //         fail!(
                //             "Could not check out branch
                // {}:\n{cant_checkout}",
                //             info.branch.local_branch_name
                //         );
                //     } else {
                //         success!(
                //             "Automatically checked out the first branch: {}",
                //             info.branch.local_branch_name
                //         );
                //     }
                // }
            },
            Err(err) => {
                fail!("{err}");
                continue;
            },
        };
    }

    Ok(())
}
