//! `branch-fetch` subcommand

use colored::Colorize as _;

use crate::cli::Remote;
use crate::commit::Commit;
use crate::git::{fetch_branch, git};

/// Fetch the given branch
pub async fn branch_fetch(
    remote: Remote,
    commit: Option<Commit>,
    checkout: bool,
) -> anyhow::Result<()> {
    match fetch_branch(&remote, commit.as_ref()).await {
        Ok((_, info)) => {
            log::info!(
                "Fetched branch {}/{}/{} available at branch {}{}",
                remote.owner,
                remote.repo,
                info.branch.upstream_branch_name,
                info.branch.local_branch_name.bright_cyan(),
                commit
                    .map(|commit_hash| {
                        format!(", at commit {}", commit_hash.as_ref().bright_yellow())
                    })
                    .unwrap_or_default()
            );

            // Attempt to cleanup after ourselves
            let _ = git(["remote", "remove", &info.remote.local_remote_alias]);

            if checkout {
                if let Err(cant_checkout) = git(["checkout", &info.branch.local_branch_name]) {
                    log::error!(
                        "Could not check out branch
                {}:\n{cant_checkout}",
                        info.branch.local_branch_name
                    );
                } else {
                    log::info!("checked out: {}", info.branch.local_branch_name);
                }
            }
        }
        Err(err) => {
            log::error!("{err}");
        }
    }

    Ok(())
}
