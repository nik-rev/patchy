use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

/// Represents the TOML config
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    pub local_branch: String,
    pub patches: IndexSet<String>,
    pub pull_requests: Vec<String>,
    pub remote_branch: String,
    pub repo: String,
}

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
pub struct BranchAndRemote {
    pub branch: Branch,
    pub remote: Remote,
}
