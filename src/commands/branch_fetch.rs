//! `branch-fetch` subcommand

use colored::Colorize as _;

use crate::config::{CommitId, Remote};
use crate::git;
use crate::github;
use anyhow::anyhow;

/// Fetch the given branch
pub async fn branch_fetch(
    remote: Remote,
    commit: Option<CommitId>,
    checkout: bool,
    use_gh_cli: bool,
) -> anyhow::Result<()> {
    let (_, info) = github::fetch_branch(&remote, use_gh_cli).await?;

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
    let _ = git::remove_remote(&info.remote.local_remote_alias);

    if checkout {
        git::checkout(info.branch.local_branch_name.as_ref()).map_err(|err| {
            anyhow!(
                "failed to check out branch {}:\n{err}",
                info.branch.local_branch_name
            )
        })?;

        log::info!("checked out: {}", info.branch.local_branch_name);
    }

    Ok(())
}
