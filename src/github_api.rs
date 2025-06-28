//! GitHub API
#![allow(
    clippy::missing_docs_in_private_items,
    reason = "GitHub API is self-explanatory"
)]

use serde::{Deserialize, Serialize};

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
    pub r#ref: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Repo {
    pub clone_url: String,
}

#[derive(Debug)]
pub struct Branch {
    pub upstream_branch_name: String,
    pub local_branch_name: String,
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
