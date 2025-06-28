//! Git commit newtype

use std::str::FromStr;

use nutype::nutype;

/// Represents a git commit
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
