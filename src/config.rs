//! Patchy's config

use anyhow::{anyhow, bail};
use itertools::Itertools;
use nutype::nutype;
use std::{convert::Infallible, str::FromStr};
use tap::Pipe as _;

use indexmap::IndexSet;
use serde::Deserialize;

/// Represents the TOML config
#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// Local branch where patchy will do all of its work
    pub local_branch: BranchName,
    /// List of patches to apply
    #[serde(default)]
    pub patches: IndexSet<String>,
    /// List of pull request to apply
    #[serde(default)]
    pub pull_requests: Vec<PullRequest>,
    /// List of branches to apply
    #[serde(default)]
    pub branches: Vec<Remote>,
    /// Branch of the remote repository
    pub remote_branch: Branch,
    /// Remote repository where all of the `branches` and `pull_requests` are
    pub repo: String,
}

/// Represents e.g. `helix-editor/helix/master @ 1a2b3c`
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Remote {
    /// e.g. `helix-editor`
    pub owner: String,
    /// e.g. `helix`
    pub repo: String,
    /// e.g. `master`
    pub branch: BranchName,
    /// e.g. `1a2b3c`
    pub commit: Option<Commit>,
}

impl Remote {
    /// Default branch for a remote
    const DEFAULT_BRANCH: &str = "main";
}

impl FromStr for Remote {
    type Err = anyhow::Error;

    /// Parse remotes of the form:
    ///
    /// ```text
    /// helix-editor/helix/master @ 1a2b3c
    /// ^^^^^^^^^^^ owner  ^^^^^^ branch
    ///              ^^^^^ repo     ^^^^^^ commit
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Ok(Ref { item, commit }) = s.parse::<Ref>();

        let mut parts = item.split('/');
        let Some([owner, repo]) = parts.next_array() else {
            bail!("Invalid branch format: {item}. Expected format: owner/repo/branch");
        };

        let branch = parts
            // insert back the removed '/', this could be part of the branch itself
            // e.g. in `helix-editor/helix/master/main` the branch is considered to be `master/main`
            //
            // NOTE: Using fully qualified syntax, as Rust will add `Iterator::intersperse`
            // in a future version.
            .pipe(|it| Itertools::intersperse(it, "/"))
            .collect::<String>()
            .pipe(|s| {
                if s.is_empty() {
                    // if branch name is ommitted (e.g. `helix-editor/helix`)
                    // then use the default branch name
                    Self::DEFAULT_BRANCH.to_string()
                } else {
                    s
                }
            })
            .pipe(BranchName::try_new)
            .map_err(|err| anyhow!("invalid branch name: {err}"))?;

        Ok(Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
            branch,
            commit,
        })
    }
}

/// Represents a pull request of a repository. E.g. `10000`, or `10000 @ deadbeef`
#[derive(Debug, Eq, PartialEq)]
pub struct PullRequest {
    /// Number of the pull request
    pub number: u32,
    /// Commit to checkout of the pull request. If none, uses the latest commit
    pub commit: Option<Commit>,
}

impl FromStr for PullRequest {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Ok(Ref {
            item: pr_number,
            commit,
        }) = s.parse::<Ref>();

        let pr_number = pr_number
            .strip_prefix('#')
            .unwrap_or(&pr_number)
            .parse::<u32>()
            .map_err(|err| anyhow!("invalid PR number: {pr_number}: {err}"))?;

        Ok(Self {
            number: pr_number,
            commit,
        })
    }
}

/// Represents a branch in git
#[derive(Eq, PartialEq, Debug)]
pub struct Branch {
    /// Name of the branch
    pub name: BranchName,
    /// Commit to checkout when fetching this branch
    pub commit: Option<Commit>,
}

impl FromStr for Branch {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Ok(Ref {
            item: branch_name,
            commit,
        }) = s.parse::<Ref>();

        Ok(Self {
            name: BranchName::try_new(branch_name)?,
            commit,
        })
    }
}

/// Represents any git item which may be associated with a commit, `<item> @ <commit>`
/// e.g. `helix-editor/helix/master @ deadbeef`
#[derive(Debug, Eq, PartialEq)]
pub struct Ref {
    /// Git item. E.g. branch, or remote which may associate with the `commit`
    pub item: String,
    /// Commit to checkout of the `item`. If none, uses the latest commit
    pub commit: Option<Commit>,
}

impl FromStr for Ref {
    type Err = Infallible;

    /// Parses user inputs of the form `<head> @ <commit-hash>`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split(" @ ").collect();

        let len = parts.len();

        if len == 1 {
            // The string does not contain the <syntax>, so the user chose to use the latest
            // commit rather than a specific one
            Self {
                item: s.into(),
                commit: None,
            }
        } else {
            // They want to use a specific commit
            let head: String = parts[0..len - 1].iter().map(|s| String::from(*s)).collect();
            let commit = (parts[len - 1].to_owned()).parse::<Commit>().ok();
            Self { item: head, commit }
        }
        .pipe(Ok)
    }
}

/// Name of a branch in git
#[nutype(
    validate(not_empty),
    derive(
        Debug, Eq, PartialEq, Ord, PartialOrd, Clone, AsRef, Display, Serialize
    )
)]
pub struct BranchName(String);

impl FromStr for BranchName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s).map_err(|err| match err {
            BranchNameError::NotEmptyViolated => "branch name cannot be empty".to_string(),
        })
    }
}

/// Represents a git commit hash
#[nutype(
    validate(not_empty, predicate = is_valid_commit_hash),
    derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, AsRef)
)]
pub struct Commit(String);

/// Does not check if the commit hash exists, just checks if it is potentially
/// valid A commit hash can consist of `a-f` and `0-9` characters
pub fn is_valid_commit_hash(hash: &str) -> bool {
    hash.chars().all(|ch| ch.is_ascii_hexdigit())
}

impl FromStr for Commit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s).map_err(|err| match err {
            CommitError::NotEmptyViolated => "commit cannot be empty".to_string(),
            CommitError::PredicateViolated => format!("invalid commit hash: {s}"),
        })
    }
}

/// Implement `Deserialize` for these types, given that they have a `FromStr` impl
// This is not a blanket impl as that would violate the orphan rule
macro_rules! impl_deserialize_for {
    ($($ty:ty)*) => {
        $(
            impl<'de> serde::Deserialize<'de> for $ty {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    String::deserialize(deserializer)?
                        .parse::<Self>()
                        .map_err(serde::de::Error::custom)
                }
            }
        )*
    };
}

impl_deserialize_for!(Remote Ref PullRequest Branch BranchName);

#[cfg(test)]
mod tests {
    use indexmap::indexset;

    use super::*;

    #[test]
    fn parse_remote() {
        let cases = [
            (
                "helix-editor/helix/master @ 1a2b3c",
                Remote {
                    owner: "helix-editor".to_string(),
                    repo: "helix".to_string(),
                    branch: BranchName::try_new("master").unwrap(),
                    commit: Some(Commit::try_new("1a2b3c".to_string()).unwrap()),
                },
            ),
            (
                "helix-editor/helix @ deadbeef",
                Remote {
                    owner: "helix-editor".to_string(),
                    repo: "helix".to_string(),
                    branch: BranchName::try_new(Remote::DEFAULT_BRANCH).unwrap(),
                    commit: Some(Commit::try_new("deadbeef".to_string()).unwrap()),
                },
            ),
            (
                "helix-editor/helix/feat/feature-x @ abc123",
                Remote {
                    owner: "helix-editor".to_string(),
                    repo: "helix".to_string(),
                    branch: BranchName::try_new("feat/feature-x").unwrap(),
                    commit: Some(Commit::try_new("abc123".to_string()).unwrap()),
                },
            ),
            (
                "owner/repo/branch",
                Remote {
                    owner: "owner".to_string(),
                    repo: "repo".to_string(),
                    branch: BranchName::try_new("branch").unwrap(),
                    commit: None,
                },
            ),
            (
                "owner/repo",
                Remote {
                    owner: "owner".to_string(),
                    repo: "repo".to_string(),
                    branch: BranchName::try_new(Remote::DEFAULT_BRANCH).unwrap(),
                    commit: None,
                },
            ),
        ];

        for (input, expected) in cases {
            let result = Remote::from_str(input);
            assert_eq!(result.unwrap(), expected, "input: {input:?}",);
        }
    }

    #[test]
    fn parse_config() {
        let config = r#"
repo = "helix-editor/helix"
remote-branch = "master @ a1b2c4"

local-branch = "patchy"

pull-requests = ["10000", "10000", "454 @ a1b2c3", "1 @ a1b2c3"]

patches = ['remove-tab']"#;

        let conf = toml::from_str::<Config>(config).unwrap();

        pretty_assertions::assert_eq!(
            conf,
            Config {
                local_branch: BranchName::try_new("patchy".to_string()).unwrap(),
                patches: indexset!["remove-tab".to_string()],
                pull_requests: vec![
                    PullRequest {
                        number: 10000,
                        commit: None
                    },
                    PullRequest {
                        number: 10000,
                        commit: None
                    },
                    PullRequest {
                        number: 454,
                        commit: Some(Commit::try_new("a1b2c3").unwrap())
                    },
                    PullRequest {
                        number: 1,
                        commit: Some(Commit::try_new("a1b2c3").unwrap())
                    },
                ],
                branches: vec![],
                remote_branch: Branch {
                    name: BranchName::try_new("master").unwrap(),
                    commit: Some(Commit::try_new("a1b2c4").unwrap())
                },
                repo: "helix-editor/helix".to_string()
            }
        );
    }
}
