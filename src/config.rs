//! Patchy's config

use anyhow::{anyhow, bail};
use itertools::Itertools;
use nutype::nutype;
use std::{convert::Infallible, fmt::Display, path::PathBuf, str::FromStr};
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
    pub patches: IndexSet<PatchName>,
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
    pub owner: RepoOwner,
    /// e.g. `helix`
    pub repo: RepoName,
    /// e.g. `master`
    pub branch: BranchName,
    /// e.g. `1a2b3c`
    pub commit: Option<CommitId>,
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

        let owner = RepoOwner::try_new(owner)?;
        let repo = RepoName::try_new(repo)?;

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
            owner,
            repo,
            branch,
            commit,
        })
    }
}

/// Represents a pull request of a repository. E.g. `10000`, or `10000 @ deadbeef`
#[derive(Debug, Eq, PartialEq)]
pub struct PullRequest {
    /// Number of the pull request
    pub number: PrNumber,
    /// Commit to checkout of the pull request. If none, uses the latest commit
    pub commit: Option<CommitId>,
}

impl FromStr for PullRequest {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Ok(Ref {
            item: pr_number,
            commit,
        }) = s.parse::<Ref>();

        let number = pr_number
            .strip_prefix('#')
            .unwrap_or(&pr_number)
            .parse()
            .map_err(|err| anyhow!("invalid PR number: {pr_number}: {err}"))?;

        Ok(Self { number, commit })
    }
}

/// Represents a branch in git
#[derive(Eq, PartialEq, Debug)]
pub struct Branch {
    /// Name of the branch
    pub name: BranchName,
    /// Commit to checkout when fetching this branch
    pub commit: Option<CommitId>,
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
    pub commit: Option<CommitId>,
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
            let commit = (parts[len - 1].to_owned()).parse::<CommitId>().ok();
            Self { item: head, commit }
        }
        .pipe(Ok)
    }
}

/// Number of a pull request
#[nutype(
    validate(greater = 0),
    derive(Eq, PartialEq, Display, Debug, FromStr, Copy, Clone, TryFrom)
)]
pub struct PrNumber(u32);

/// Represents owner of a repository
///
/// E.g. in `helix-editor/helix/master`, this is `helix-editor`
#[nutype(
    validate(not_empty),
    derive(
        Debug, Eq, PartialEq, Ord, PartialOrd, Clone, AsRef, Display, Serialize, TryFrom
    )
)]
pub struct RepoOwner(String);

/// Represents name of a repository
///
/// E.g. in `helix-editor/helix/master`, this is `helix`
#[nutype(
    validate(not_empty),
    derive(
        Debug, Eq, PartialEq, Ord, PartialOrd, Clone, AsRef, Display, Serialize, TryFrom
    )
)]
pub struct RepoName(String);

/// Name of a branch in git
///
/// E.g. in `helix-editor/helix/master`, this is `master`
#[nutype(
    validate(not_empty),
    derive(
        Debug, Eq, PartialEq, Ord, PartialOrd, Clone, AsRef, Display, Serialize, TryFrom
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

/// File name of a patch
#[nutype(
    validate(predicate = |p| !p.as_os_str().is_empty()),
    derive(Hash, Eq, PartialEq, Debug, AsRef, Deserialize, Clone, FromStr, TryFrom)
)]
pub struct PatchName(PathBuf);

impl TryFrom<&str> for PatchName {
    type Error = PatchNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        PatchName::try_new(PathBuf::from(value))
    }
}

impl Display for PatchName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref().display())
    }
}

/// Represents a git commit hash
#[nutype(
    validate(not_empty, predicate = is_valid_commit_hash),
    derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, AsRef, TryFrom, FromStr, Display)
)]
pub struct CommitId(String);

/// Does not check if the commit exists, just checks if it is potentially valid
///
/// A commit hash can consist of `a-f` and `0-9` characters
pub fn is_valid_commit_hash(hash: &str) -> bool {
    hash.chars().all(|ch| ch.is_ascii_hexdigit())
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
                    owner: "helix-editor".try_into().unwrap(),
                    repo: "helix".try_into().unwrap(),
                    branch: "master".try_into().unwrap(),
                    commit: Some("1a2b3c".try_into().unwrap()),
                },
            ),
            (
                "helix-editor/helix @ deadbeef",
                Remote {
                    owner: "helix-editor".try_into().unwrap(),
                    repo: "helix".try_into().unwrap(),
                    branch: Remote::DEFAULT_BRANCH.try_into().unwrap(),
                    commit: Some("deadbeef".try_into().unwrap()),
                },
            ),
            (
                "helix-editor/helix/feat/feature-x @ abc123",
                Remote {
                    owner: "helix-editor".try_into().unwrap(),
                    repo: "helix".try_into().unwrap(),
                    branch: "feat/feature-x".try_into().unwrap(),
                    commit: Some("abc123".try_into().unwrap()),
                },
            ),
            (
                "owner/repo/branch",
                Remote {
                    owner: "owner".try_into().unwrap(),
                    repo: "repo".try_into().unwrap(),
                    branch: "branch".try_into().unwrap(),
                    commit: None,
                },
            ),
            (
                "owner/repo",
                Remote {
                    owner: "owner".try_into().unwrap(),
                    repo: "repo".try_into().unwrap(),
                    branch: Remote::DEFAULT_BRANCH.try_into().unwrap(),
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
                local_branch: "patchy".try_into().unwrap(),
                patches: indexset!["remove-tab".try_into().unwrap()],
                pull_requests: vec![
                    PullRequest {
                        number: 10000.try_into().unwrap(),
                        commit: None
                    },
                    PullRequest {
                        number: 10000.try_into().unwrap(),
                        commit: None
                    },
                    PullRequest {
                        number: 454.try_into().unwrap(),
                        commit: Some("a1b2c3".try_into().unwrap())
                    },
                    PullRequest {
                        number: 1.try_into().unwrap(),
                        commit: Some("a1b2c3".try_into().unwrap())
                    },
                ],
                branches: vec![],
                remote_branch: Branch {
                    name: "master".try_into().unwrap(),
                    commit: Some("a1b2c4".try_into().unwrap())
                },
                repo: "helix-editor/helix".to_string()
            }
        );
    }
}
