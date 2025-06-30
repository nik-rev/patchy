//! GitHub API
#![allow(
    clippy::missing_docs_in_private_items,
    reason = "GitHub API is self-explanatory"
)]

use std::process;

use serde::{Deserialize, Serialize};
use tap::Pipe as _;

use crate::{
    config::{BranchName, CommitId, PrNumber},
    git_high_level::{AvailableBranch, add_remote_branch, find_first_available_branch},
    utils::{make_request, normalize_commit_msg, with_uuid},
};
use anyhow::{Result, anyhow};

/// Data returned by GitHub's API
#[derive(Serialize, Deserialize, Debug)]
pub struct GitHubResponse {
    pub head: Head,
    pub title: String,
    pub html_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Head {
    pub repo: Repo,
    pub r#ref: BranchName,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Repo {
    pub clone_url: String,
}

#[derive(Debug)]
pub struct Branch {
    pub upstream_branch_name: BranchName,
    pub local_branch_name: BranchName,
}

#[derive(Debug)]
pub struct Remote {
    pub repository_url: String,
    pub local_remote_alias: String,
}

#[derive(Debug)]
pub struct RemoteBranch {
    pub remote: Remote,
    pub branch: Branch,
}

/// Make a request to GitHub's API.
///
/// Either manually fetch the URL or use `gh` CLI
async fn gh_api(url: &str, use_gh_cli: bool) -> Result<String> {
    if use_gh_cli {
        process::Command::new("gh")
            .arg("api")
            .arg(url)
            .output()?
            .stdout
            .pipe(String::from_utf8)?
            .pipe(Ok)
    } else {
        make_request(url).await
    }
}

/// Fetch the branch of `remote` at the given `commit`
pub async fn fetch_branch(
    remote: &crate::config::Remote,
    use_gh_cli: bool,
) -> Result<(Repo, RemoteBranch)> {
    let owner = &remote.owner;
    let repo = &remote.repo;
    let url = format!("https://api.github.com/repos/{owner}/{repo}",);

    let response = gh_api(&url, use_gh_cli)
        .await
        .map_err(|err| anyhow!("failed to fetch branch `{owner}/{repo}`:\n{err}\n"))?;

    let response: Repo = serde_json::from_str(&response).map_err(|err| {
        anyhow!("failed to parse response.\n{response}. failed to parse because: \n{err}")
    })?;

    let info = RemoteBranch {
        remote: Remote {
            repository_url: response.clone_url.clone(),
            local_remote_alias: with_uuid(&format!("{}/{}", &remote.owner, remote.repo)),
        },
        branch: Branch {
            local_branch_name: remote.branch.clone(),
            upstream_branch_name: remote.branch.clone(),
        },
    };

    add_remote_branch(&info, remote.commit.as_ref()).map_err(|err| {
        anyhow!(
            "Could not add remote branch {}/{}, skipping.\n{err}",
            remote.owner,
            remote.repo
        )
    })?;

    Ok((response, info))
}

/// Fetch PR `pull_request` at `commit_hash` from `repo` to a local `custom_branch_name`,
/// the branch name is generated if not supplied
pub async fn fetch_pull_request(
    repo: &str,
    pull_request: PrNumber,
    custom_branch_name: Option<BranchName>,
    commit_hash: Option<&CommitId>,
    use_gh_cli: bool,
) -> Result<(GitHubResponse, RemoteBranch)> {
    let url = format!("https://api.github.com/repos/{repo}/pulls/{pull_request}");

    let gh_response = if use_gh_cli {
        process::Command::new("gh")
            .arg("api")
            .arg(url)
            .output()?
            .stdout
            .pipe(String::from_utf8)?
    } else {
        make_request(&url)
            .await
            .map_err(|err| anyhow!("failed to fetch pull request #{pull_request}\n{err}\n"))?
    };

    let response: GitHubResponse = serde_json::from_str(&gh_response).map_err(|err| {
        anyhow!("failed to parse GitHub response.\n{gh_response}. Could not parse because: \n{err}")
    })?;

    let remote_branch = RemoteBranch {
        remote: Remote {
            repository_url: response.head.repo.clone_url.clone(),
            local_remote_alias: with_uuid(&format!(
                "{title}-{}",
                pull_request,
                title = normalize_commit_msg(&response.html_url)
            )),
        },
        branch: Branch {
            upstream_branch_name: response.head.r#ref.clone(),
            local_branch_name: custom_branch_name.map_or_else(
                || {
                    let branch_name = &format!("{pull_request}/{}", &response.head.r#ref);

                    match find_first_available_branch(branch_name) {
                        AvailableBranch::First => BranchName::try_new(branch_name)
                            .expect("name of the branch we create is valid"),
                        AvailableBranch::Other(branch) => branch,
                    }
                },
                Into::into,
            ),
        },
    };

    add_remote_branch(&remote_branch, commit_hash).map_err(|err| {
        anyhow!("failed to add remote branch for pull request #{pull_request}, skipping.\n{err}")
    })?;

    Ok((response, remote_branch))
}
