//! `pr-fetch` subcommand

use anyhow::{Context as _, anyhow};
use colored::Colorize as _;

use crate::config::{BranchName, CommitId, PrNumber, Remote, RepoName, RepoOwner};
use crate::{git, github};

/// Fetch the given `pr` of `remote` at `commit` and store it in local `branch`
///
/// If `checkout`, `--checkout` the `branch`
pub async fn pr_fetch(
    pr: PrNumber,
    remote: Option<Remote>,
    branch: Option<BranchName>,
    commit: Option<CommitId>,
    checkout: bool,
    use_gh_cli: bool,
) -> anyhow::Result<()> {
    pub const GITHUB_REMOTE_PREFIX: &str = "git@github.com:";
    pub const GITHUB_REMOTE_SUFFIX: &str = ".git";

    // The user hasn't provided a custom remote, so we're going to try `origin`
    let remote = remote.map_or_else(
        || -> anyhow::Result<Remote> {
            let remote = git::get_remote_url("origin")?;
            let err = || anyhow!("git command returned invalid remote: {remote}");

            if remote.starts_with(GITHUB_REMOTE_PREFIX) && remote.ends_with(GITHUB_REMOTE_SUFFIX) {
                let start = GITHUB_REMOTE_PREFIX.len();
                let end = remote.len() - GITHUB_REMOTE_SUFFIX.len();
                let (owner, repo) = remote
                    .get(start..end)
                    .and_then(|x| x.split_once('/'))
                    .with_context(err)?;
                Ok(Remote {
                    owner: RepoOwner::try_new(owner)?,
                    repo: RepoName::try_new(repo)?,
                    branch: BranchName::try_new("main").expect("`main` is a valid branch name"),
                    commit: None,
                })
            } else {
                Err(err())
            }
        },
        Ok,
    )?;

    let Ok((response, info)) = github::fetch_pull_request(
        &format!("{}/{}", remote.owner, remote.repo),
        pr,
        branch,
        commit.as_ref(),
        use_gh_cli,
    )
    .await
    .inspect_err(|err| {
        log::error!("{err}");
    }) else {
        return Ok(());
    };

    log::info!(
        "Fetched pull request {} available at branch {}{}",
        crate::utils::format_pr(pr, &response.title, &response.html_url),
        info.branch.local_branch_name.as_ref().bright_cyan(),
        commit
            .clone()
            .map(|commit_hash| { format!(", at commit {}", commit_hash.as_ref().bright_yellow()) })
            .unwrap_or_default()
    );

    // Attempt to cleanup after ourselves
    let _ = git::remove_remote(&info.remote.local_remote_alias);

    if checkout {
        if let Err(checkout_err) = git::checkout(info.branch.local_branch_name.as_ref()) {
            log::error!(
                "Could not check out branch {}:\n{checkout_err}",
                info.branch.local_branch_name
            );
        } else {
            log::info!(
                "Automatically checked out the first branch: {}",
                info.branch.local_branch_name
            );
        }
    }

    Ok(())
}
