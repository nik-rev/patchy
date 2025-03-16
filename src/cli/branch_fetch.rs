use super::{CliParseError, GlobalFlag, LocalFlag, SubCommand};

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Branch {
    /// Name of this branch in the remote
    pub name: String,
    /// When fetching this PR, reset to this commit
    pub commit: Option<String>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BranchFetch {
    pub branches: Vec<Branch>,
}

impl SubCommand for BranchFetch {
    fn parse<I: Iterator<Item = String>>(
        args: &mut I,
        global_flag: &mut GlobalFlag,
    ) -> Result<Self, CliParseError> {
        let mut branches: Vec<Branch> = vec![];

        for arg in args.by_ref() {
            if let Ok(flag) = arg.parse::<GlobalFlag>() {
                global_flag.validate(flag)?;
                continue;
            }

            // Non-flag arguments for branch-fetch are always branch names with optional
            // commits
            if let Some(local_flag) = LocalFlag::parse(&arg)? {
                // Only global flags should be parsed for branch-fetch
                return Err(CliParseError::UnexpectedFlag(local_flag));
            }

            let (branch_name, commit) = match arg.split_once('@') {
                Some((branch_name, commit)) => {
                    if commit.is_empty() {
                        return Err(CliParseError::EmptyArgument(format!(
                            "commit is empty for {arg}"
                        )));
                    };
                    (branch_name, Some(commit))
                },
                None => (arg.as_str(), None),
            };

            branches.push(Branch {
                name: branch_name.to_owned(),
                commit: commit.map(ToOwned::to_owned),
            });
        }

        Ok(BranchFetch { branches })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cli::{Cli, Subcommand, patchy};

    #[test]
    fn single_branch() {
        assert_eq!(
            patchy(&["branch-fetch", "helix-editor/helix/master"]),
            Ok(Cli {
                subcommand: Some(Subcommand::BranchFetch(BranchFetch {
                    branches: vec![Branch {
                        name: "helix-editor/helix/master".to_owned(),
                        commit: None,
                    }],
                })),
                global_flag: GlobalFlag::None,
            })
        );
    }

    #[test]
    fn multiple_branches() {
        assert_eq!(
            patchy(&[
                "branch-fetch",
                "helix-editor/helix/master",
                "helix-editor/helix/develop"
            ]),
            Ok(Cli {
                subcommand: Some(Subcommand::BranchFetch(BranchFetch {
                    branches: vec![
                        Branch {
                            name: "helix-editor/helix/master".to_owned(),
                            commit: None,
                        },
                        Branch {
                            name: "helix-editor/helix/develop".to_owned(),
                            commit: None,
                        }
                    ],
                })),
                global_flag: GlobalFlag::None,
            })
        );
    }

    #[test]
    fn specific_commit() {
        assert_eq!(
            patchy(&["branch-fetch", "helix-editor/helix/master@6049f20"]),
            Ok(Cli {
                subcommand: Some(Subcommand::BranchFetch(BranchFetch {
                    branches: vec![Branch {
                        name: "helix-editor/helix/master".to_owned(),
                        commit: Some("6049f20".to_owned()),
                    }],
                })),
                global_flag: GlobalFlag::None,
            })
        );
    }

    #[test]
    fn many_branches_some_with_commits() {
        assert_eq!(
            patchy(&[
                "branch-fetch",
                "helix-editor/helix/master@6049f20",
                "helix-editor/helix/develop",
                "helix-editor/helix/feature@abc123"
            ]),
            Ok(Cli {
                subcommand: Some(Subcommand::BranchFetch(BranchFetch {
                    branches: vec![
                        Branch {
                            name: "helix-editor/helix/master".to_owned(),
                            commit: Some("6049f20".to_owned()),
                        },
                        Branch {
                            name: "helix-editor/helix/develop".to_owned(),
                            commit: None,
                        },
                        Branch {
                            name: "helix-editor/helix/feature".to_owned(),
                            commit: Some("abc123".to_owned()),
                        }
                    ],
                })),
                global_flag: GlobalFlag::None,
            })
        );
    }

    #[test]
    fn multiple_at_in_branch_name() {
        assert_eq!(
            patchy(&["branch-fetch", "repo/branch@commit@extra"]),
            Ok(Cli {
                subcommand: Some(Subcommand::BranchFetch(BranchFetch {
                    branches: vec![Branch {
                        name: "repo/branch".to_owned(),
                        commit: Some("commit@extra".to_owned()),
                    },],
                })),
                global_flag: GlobalFlag::None,
            })
        );
    }

    #[test]
    fn with_global_flags() {
        assert_eq!(
            patchy(&["branch-fetch", "--help"]),
            Ok(Cli {
                subcommand: Some(Subcommand::BranchFetch(BranchFetch { branches: vec![] })),
                global_flag: GlobalFlag::Help,
            })
        );

        assert_eq!(
            patchy(&["branch-fetch", "--version"]),
            Ok(Cli {
                subcommand: Some(Subcommand::BranchFetch(BranchFetch { branches: vec![] })),
                global_flag: GlobalFlag::Version,
            })
        );
    }

    #[test]
    fn invalid_flags() {
        assert_eq!(
            patchy(&["branch-fetch", "--yes"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::Yes))
        );
        assert_eq!(
            patchy(&["branch-fetch", "--checkout"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::Checkout))
        );
        assert_eq!(
            patchy(&["branch-fetch", "--branch-name=test"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::BranchName(
                "test".to_owned()
            )))
        );
        assert_eq!(
            patchy(&["branch-fetch", "--repo-name=test"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::RepoName(
                "test".to_owned()
            )))
        );
        assert_eq!(
            patchy(&["branch-fetch", "--patch-filename=test"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::PatchFilename(
                "test".to_owned()
            )))
        );
    }
}
