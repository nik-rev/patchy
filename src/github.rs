//! GitHub API

use std::process;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tap::Pipe as _;

use crate::{
    config::{BranchName, CommitId, PrNumber, RepoName, RepoOwner},
    git,
    utils::{make_request, normalize_commit_msg, with_uuid},
};
use anyhow::{Result, anyhow, bail};

/// Data returned by GitHub's API for the pull request endpoint per repo
#[derive(Serialize, Deserialize, Debug)]
pub struct PrData {
    /// Data about the head repository
    pub head: Head,
    /// Title of the pull request
    pub title: String,
    /// Url to the pull request
    pub html_url: String,
}

/// Head repository (returned by github api)
#[derive(Serialize, Deserialize, Debug)]
pub struct Head {
    /// Repo for the PR
    pub repo: Repo,
    /// Name of the branch of the PR
    pub r#ref: BranchName,
}

impl PrData {
    /// The endpoint which returns the structure `GitHubResponse`
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

/// Branch
#[derive(Debug)]
pub struct Branch {
    /// Name of the branch as it is on the remote
    pub upstream_branch_name: BranchName,
    /// Name of the branch when we want to clone it locally
    pub local_branch_name: BranchName,
}

/// Remote
#[derive(Debug)]
pub struct Remote {
    /// Link to the remote repository
    pub repository_url: String,
    /// Name of the remote as it exists locally
    pub local_remote_alias: String,
}

/// Associates a remote with a branch
#[derive(Debug)]
pub struct RemoteBranch {
    /// Remote
    pub remote: Remote,
    /// Branch
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
) -> Result<(PrData, RemoteBranch)> {
    let url = PrData::endpoint(repo, pull_request);

    let response = get_gh_api::<PrData>(&url, use_gh_cli)
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

/// Available branch name to use
pub enum AvailableBranch {
    /// In this case, we can just use the original `branch` that we passed in
    First,
    /// The first branch was available, so we slapped on some arbitrary
    /// identifier at the end Represents a branch like some-branch-2,
    /// some-branch-3
    Other(BranchName),
}

/// Given a branch, either return this branch or the first available branch with
/// an identifier at the end (a `-#`) where `#` represents a number
/// So we can keep on "trying" for a branch that isn't used. We might try
/// `some-branch`, and if it already exists we will then try:
///
/// - some-branch-2
/// - some-branch-3
/// - some-branch-4
/// - ...
///
/// Stopping when we find the first available
///
/// We do not want to return a branch if it already exists, since we don't want
/// to overwrite any branch potentially losing the user their work
///
/// We also don't want to ask for a prompt for a custom name, as it would be
/// pretty annoying to specify a name for each branch if you have like 30 pull
/// requests you want to merge
pub fn find_first_available_branch(branch: &str) -> AvailableBranch {
    if git::does_object_exist(branch) {
        return AvailableBranch::First;
    }

    // the first number for which the branch does not exist
    #[expect(
        clippy::maybe_infinite_iter,
        reason = "there is definitely not an infinite number of branches"
    )]
    let number = (2..)
        .find(|current| git::does_object_exist(&format!("{current}-{branch}")))
        .expect("There will eventually be a #-branch which is available.");

    let branch_name = BranchName::try_new(format!("{number}-{branch}"))
        .expect("existing git branch is a valid branch name");

    AvailableBranch::Other(branch_name)
}

/// Fetches a branch of a remote into local. Optionally accepts a commit hash
/// for versioning.
pub fn add_remote_branch(remote_branch: &RemoteBranch, commit: Option<&CommitId>) -> Result<()> {
    git::add_remote(
        &remote_branch.remote.local_remote_alias,
        &remote_branch.remote.repository_url,
    )
    .map_err(|err| anyhow!("failed to fetch remote: {err}"))?;

    if let Err(err) = git::fetch_remote_branch(
        &remote_branch.branch.local_branch_name,
        &remote_branch.branch.upstream_branch_name,
        &remote_branch.remote.repository_url,
    ) {
        bail!(
            "Failed to find branch {} of GitHub repository {}. Are you sure it exists?\n{err}",
            remote_branch.branch.upstream_branch_name,
            remote_branch.remote.repository_url
        );
    }

    if let Some(commit) = commit {
        git::reset_branch_to_commit(&remote_branch.branch.local_branch_name, commit).map_err(
            |err| {
                anyhow!(
                    "Failed to find commit {} of branch {}. Are you sure the commit exists?\n{err}",
                    commit.as_ref(),
                    remote_branch.branch.local_branch_name
                )
            },
        )?;
    }

    Ok(())
}
