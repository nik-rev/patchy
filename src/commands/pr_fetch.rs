use anyhow::{Context as _, anyhow};
use colored::Colorize as _;

use crate::cli::Remote;
use crate::commit::Commit;
use crate::git::{fetch_pull_request, git};
use crate::utils::display_link;
use crate::{fail, success};

/// Allow users to prefix their PRs with octothorpe, e.g. #12345 instead of
/// 12345. This is just a QOL addition since some people may use it due to habit
pub fn ignore_octothorpe(arg: &str) -> String {
    if arg.starts_with('#') {
        arg.get(1..).unwrap_or_default()
    } else {
        arg
    }
    .into()
}

pub async fn pr_fetch(
    pr: u32,
    remote: Option<Remote>,
    branch: Option<String>,
    commit: Option<Commit>,
    checkout: bool,
) -> anyhow::Result<()> {
    pub const GITHUB_REMOTE_PREFIX: &str = "git@github.com:";
    pub const GITHUB_REMOTE_SUFFIX: &str = ".git";

    // The user hasn't provided a custom remote, so we're going to try `origin`
    let remote = remote.map_or_else(
        || -> anyhow::Result<Remote> {
            let remote = git(["remote", "get-url", "origin"])?;
            let err = || anyhow!("git command returned invalid remote. Output {remote}");
            if remote.starts_with(GITHUB_REMOTE_PREFIX) && remote.ends_with(GITHUB_REMOTE_SUFFIX) {
                let start = GITHUB_REMOTE_PREFIX.len();
                let end = remote.len() - GITHUB_REMOTE_SUFFIX.len();
                let (owner, repo) = remote
                    .get(start..end)
                    .and_then(|x| x.split_once('/'))
                    .with_context(err)?;
                Ok(Remote {
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                    branch: "main".to_string(),
                })
            } else {
                Err(err())
            }
        },
        Ok,
    )?;

    match fetch_pull_request(
        &format!("{}/{}", remote.owner, remote.repo),
        // TODO: make fetch_pull_request accept a u32 instead
        &pr.to_string(),
        branch.as_deref(),
        commit.as_ref(),
    )
    .await
    {
        Ok((response, info)) => {
            success!(
                "Fetched pull request {} available at branch {}{}",
                display_link(
                    &format!(
                        "{}{}{}{}",
                        "#".bright_blue(),
                        pr.to_string().bright_blue(),
                        " ".bright_blue(),
                        response.title.bright_blue().italic()
                    ),
                    &response.html_url
                ),
                info.branch.local_branch_name.bright_cyan(),
                commit
                    .clone()
                    .map(|commit_hash| {
                        format!(", at commit {}", commit_hash.as_ref().bright_yellow())
                    })
                    .unwrap_or_default()
            );

            // Attempt to cleanup after ourselves
            let _ = git(["remote", "remove", &info.remote.local_remote_alias]);

            if checkout {
                if let Err(cant_checkout) = git(["checkout", &info.branch.local_branch_name]) {
                    fail!(
                        "Could not check out branch {}:\n{cant_checkout}",
                        info.branch.local_branch_name
                    );
                } else {
                    success!(
                        "Automatically checked out the first branch: {}",
                        info.branch.local_branch_name
                    );
                }
            }
        }
        Err(err) => {
            fail!("{err}");
        }
    }

    Ok(())
}
