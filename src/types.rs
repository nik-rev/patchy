use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

pub type CommandArgs = IndexSet<String>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    pub local_branch: String,
    pub patches: IndexSet<String>,
    pub pull_requests: Vec<String>,
    pub remote_branch: String,
    pub repo: String,
}

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

impl BranchAndRemote {
    pub fn new(
        local_branch: &str,
        remote_branch: &str,
        local_remote: &str,
        remote_remote: &str,
    ) -> Self {
        let branch = Branch {
            local_branch_name: local_branch.into(),
            upstream_branch_name: remote_branch.into(),
        };
        let remote = Remote {
            local_remote_alias: local_remote.into(),
            repository_url: remote_remote.into(),
        };
        Self { branch, remote }
    }
}
