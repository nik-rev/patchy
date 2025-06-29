//! `branch-fetch` subcommand

use colored::Colorize as _;

use crate::config::{CommitId, Remote};
use crate::git::{fetch_branch, git};
use anyhow::anyhow;

/// Fetch the given branch
pub async fn branch_fetch(
    remote: Remote,
    commit: Option<CommitId>,
    checkout: bool,
) -> anyhow::Result<()> {
    let (_, info) = fetch_branch(&remote).await?;

    log::info!(
        "Fetched branch {}/{}/{} available at branch {}{}",
        remote.owner,
        remote.repo,
        info.branch.upstream_branch_name,
        info.branch.local_branch_name.as_ref().bright_cyan(),
        commit
            .map(|commit_hash| { format!(", at commit {}", commit_hash.as_ref().bright_yellow()) })
            .unwrap_or_default()
    );

    // Attempt to cleanup after ourselves
    let _ = git(["remote", "remove", &info.remote.local_remote_alias]);

    if checkout {
        git(["checkout", info.branch.local_branch_name.as_ref()]).map_err(|err| {
            anyhow!(
                "failed to check out branch {}:\n{err}",
                info.branch.local_branch_name
            )
        })?;

        log::info!("checked out: {}", info.branch.local_branch_name);
    }

    Ok(())
}
