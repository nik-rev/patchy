use documented::{Documented, DocumentedFields};

use super::flags::CliFlag;
use super::{CliParseError, Flag, HelpOrVersion, LocalFlag, SubCommand};
use crate::git_commands::Commit;

/// Represents a single branch
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Documented, DocumentedFields)]
pub struct Branch {
    /// Name of the GitHub owner of the repository
    pub repo_owner: String,
    /// Name of the repository this branch belongs to
    pub repo_name: String,
    /// Name of this branch in the remote
    pub name: String,
    /// When fetching this PR, reset to this commit
    pub commit: Option<Commit>,
}

/// Fetch branches for a GitHub repository as a local branch
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Documented)]
pub struct BranchFetch {
    /// A list of branches to fetch
    pub branches: Vec<Branch>,
    pub checkout: bool,
}

impl BranchFetch {
    pub const CHECKOUT_FLAG: CliFlag<'static> = CliFlag {
        short: "-c",
        long: "--checkout",
        description: "Check out the first fetched branch",
    };

    pub const BRANCH_NAME_FLAG: CliFlag<'static> = CliFlag {
        short: "-b=",
        long: "--branch-name=",
        description: "Choose local name for the branch belonging to the preceding pull request",
    };
}

impl SubCommand for BranchFetch {
    const NAME: &str = "branch-fetch";

    fn parse<I: Iterator<Item = String>>(
        args: &mut I,
        global_flag: &mut HelpOrVersion,
    ) -> Result<Self, CliParseError> {
        let mut branches: Vec<Branch> = vec![];
        let mut checkout = false;

        for arg in args.by_ref() {
            if let Ok(flag) = arg.parse::<HelpOrVersion>() {
                global_flag.validate(flag)?;
                continue;
            }

            match LocalFlag::parse(&arg)? {
                Some(flag @ LocalFlag::Checkout) => {
                    if checkout {
                        return Err(CliParseError::DuplicateFlag(Flag::LocalFlag(flag)));
                    }
                    checkout = true;
                    continue;
                },
                Some(flag) => return Err(CliParseError::UnexpectedFlag(flag)),
                None => (),
            }

            let (branch_name, commit) = match arg.split_once('@') {
                Some((branch_name, commit)) => {
                    if commit.is_empty() {
                        return Err(CliParseError::EmptyArgument(format!(
                            "commit is empty for {arg}"
                        )));
                    }
                    (branch_name, Some(commit))
                },
                None => (arg.as_str(), None),
            };

            let commit = commit.map(|s| Commit::parse(s.to_owned())).transpose()?;

            let Some((repo_owner, repo_name_and_branch_name)) = branch_name.split_once('/') else {
                return Err(CliParseError::InvalidRepo(branch_name.to_owned()));
            };

            let Some((repo_name, branch_name)) = repo_name_and_branch_name.split_once('/') else {
                return Err(CliParseError::InvalidRepo(branch_name.to_owned()));
            };

            branches.push(Branch {
                repo_owner: repo_owner.to_owned(),
                repo_name: repo_name.to_owned(),
                name: branch_name.to_owned(),
                commit,
            });
        }

        if checkout && branches.is_empty() {
            return Err(CliParseError::CheckoutNoSource);
        }

        Ok(BranchFetch { branches, checkout })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cli::tests::patchy;
    use crate::cli::{Cli, Subcommand};

    #[test]
    fn single_branch() {
        assert_eq!(
            patchy(&["branch-fetch", "helix-editor/helix/master"]),
            Ok(Cli {
                subcommand: Some(Subcommand::BranchFetch(BranchFetch {
                    branches: vec![Branch {
                        repo_owner: "helix-editor".to_owned(),
                        repo_name: "helix".to_owned(),
                        name: "master".to_owned(),
                        commit: None,
                    }],
                    checkout: false,
                })),
                help_or_version: HelpOrVersion::None,
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
                            repo_owner: "helix-editor".to_owned(),
                            repo_name: "helix".to_owned(),
                            name: "master".to_owned(),
                            commit: None,
                        },
                        Branch {
                            repo_owner: "helix-editor".to_owned(),
                            repo_name: "helix".to_owned(),
                            name: "develop".to_owned(),
                            commit: None,
                        }
                    ],
                    checkout: false,
                })),
                help_or_version: HelpOrVersion::None,
            })
        );
        // with checkout flag
        assert_eq!(
            patchy(&[
                "branch-fetch",
                "-c",
                "helix-editor/helix/master",
                "helix-editor/helix/develop"
            ]),
            Ok(Cli {
                subcommand: Some(Subcommand::BranchFetch(BranchFetch {
                    branches: vec![
                        Branch {
                            repo_owner: "helix-editor".to_owned(),
                            repo_name: "helix".to_owned(),
                            name: "master".to_owned(),
                            commit: None,
                        },
                        Branch {
                            repo_owner: "helix-editor".to_owned(),
                            repo_name: "helix".to_owned(),
                            name: "develop".to_owned(),
                            commit: None,
                        }
                    ],
                    checkout: true,
                })),
                help_or_version: HelpOrVersion::None,
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
                        repo_owner: "helix-editor".to_owned(),
                        repo_name: "helix".to_owned(),
                        name: "master".to_owned(),
                        commit: Some(Commit::parse("6049f20".to_owned()).unwrap()),
                    }],
                    checkout: false,
                })),
                help_or_version: HelpOrVersion::None,
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
                            repo_owner: "helix-editor".to_owned(),
                            repo_name: "helix".to_owned(),
                            name: "master".to_owned(),
                            commit: Some(Commit::parse("6049f20".to_owned()).unwrap()),
                        },
                        Branch {
                            repo_owner: "helix-editor".to_owned(),
                            repo_name: "helix".to_owned(),
                            name: "develop".to_owned(),
                            commit: None,
                        },
                        Branch {
                            repo_owner: "helix-editor".to_owned(),
                            repo_name: "helix".to_owned(),
                            name: "feature".to_owned(),
                            commit: Some(Commit::parse("abc123".to_owned()).unwrap()),
                        }
                    ],
                    checkout: false,
                })),
                help_or_version: HelpOrVersion::None,
            })
        );
    }

    #[test]
    fn with_global_flags() {
        assert_eq!(
            patchy(&["branch-fetch", "--help"]),
            Ok(Cli {
                subcommand: Some(Subcommand::BranchFetch(BranchFetch {
                    branches: vec![],
                    checkout: false
                })),
                help_or_version: HelpOrVersion::Help,
            })
        );

        assert_eq!(
            patchy(&["branch-fetch", "--version"]),
            Ok(Cli {
                subcommand: Some(Subcommand::BranchFetch(BranchFetch {
                    branches: vec![],
                    checkout: false
                })),
                help_or_version: HelpOrVersion::Version,
            })
        );
    }

    #[test]
    fn duplicate_checkout() {
        assert_eq!(
            patchy(&["branch-fetch", "some/branch/somewhere", "-c", "--checkout"]),
            Err(CliParseError::DuplicateFlag(Flag::LocalFlag(
                LocalFlag::Checkout
            )))
        );
    }

    #[test]
    fn checkout_with_no_source() {
        assert_eq!(
            patchy(&["branch-fetch", "-c"]),
            Err(CliParseError::CheckoutNoSource)
        );
        assert_eq!(
            patchy(&["branch-fetch", "--checkout"]),
            Err(CliParseError::CheckoutNoSource)
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
            Err(CliParseError::CheckoutNoSource)
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
