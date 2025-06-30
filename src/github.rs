//! GitHub API
#![allow(
    clippy::missing_docs_in_private_items,
    reason = "GitHub API is self-explanatory"
)]

use std::process;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tap::Pipe as _;

use crate::{
    config::{BranchName, CommitId, PrNumber, RepoName, RepoOwner},
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

impl GitHubResponse {
    /// The endpoint which returns the structure [`GitHubResponse`]
    fn endpoint(repo: &str, pull_request: PrNumber) -> String {
        format!("https://api.github.com/repos/{repo}/pulls/{pull_request}")
    }
}

/// Data returned by endpoint
#[derive(Serialize, Deserialize, Debug)]
pub struct Repo {
    /// e.g. `https://github.com/helix-editor/helix.git`
    pub clone_url: String,
}

impl Repo {
    /// the endpoint that returns the structure [`Repo`]
    pub fn endpoint(owner: &RepoOwner, repo: &RepoName) -> String {
        format!("https://api.github.com/repos/{owner}/{repo}",)
    }
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

/////////////////////////////////////////////////////////

/// Make a request to GitHub's API.
///
/// Either manually fetch the URL or use `gh` CLI
///
/// - Outer `Result`: Failed to fetch the URL
/// - Inner `Result`: Failed to deserialize text received by the URL
async fn get_gh_api<T: DeserializeOwned>(url: &str, use_gh_cli: bool) -> Result<Result<T>> {
    log::trace!("making a request to {url}");
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
    .map(|response| {
        serde_json::from_str::<T>(&response).map_err(|err| {
            anyhow!("failed to parse response.\n{response}. failed to parse because: \n{err}")
        })
    })
}

/// Fetch the branch of `remote` at the given `commit`
pub async fn fetch_branch(
    remote: &crate::config::Remote,
    use_gh_cli: bool,
) -> Result<(Repo, RemoteBranch)> {
    let owner = &remote.owner;
    let repo = &remote.repo;
    let url = Repo::endpoint(owner, repo);

    let response = get_gh_api::<Repo>(&url, use_gh_cli)
        .await
        .map_err(|err| anyhow!("failed to fetch branch `{owner}/{repo}`:\n{err}\n"))??;

    let info = RemoteBranch {
        remote: Remote {
            repository_url: response.clone_url.clone(),
            local_remote_alias: with_uuid(&format!("{}/{}", &owner, repo)),
        },
        branch: Branch {
            local_branch_name: remote.branch.clone(),
            upstream_branch_name: remote.branch.clone(),
        },
    };

    add_remote_branch(&info, remote.commit.as_ref()).map_err(|err| {
        anyhow!(
            "Could not add remote branch {}/{}, skipping.\n{err}",
            owner,
            repo
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
    let url = GitHubResponse::endpoint(repo, pull_request);

    let response = get_gh_api::<GitHubResponse>(&url, use_gh_cli)
        .await
        .map_err(|err| anyhow!("failed to fetch pull request #{pull_request}\n{err}\n"))??;

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
