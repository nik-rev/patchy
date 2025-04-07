use documented::{Documented, DocumentedFields};

use super::flags::CliFlag;
use super::{CliParseError, Flag, HelpOrVersion, LocalFlag, SubCommand};
use crate::git_commands::Commit;

/// A pull request
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Documented, DocumentedFields)]
pub struct Pr {
    /// Fetch PR of this number
    pub number: u32,
    /// When fetching this PR, reset to this commit
    pub commit: Option<Commit>,
    /// Choose local name for the branch belonging to the preceding pull request
    pub custom_branch_name: Option<String>,
}

/// Fetch pull request for a GitHub repository as a local branch
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Documented, DocumentedFields)]
pub struct PrFetch {
    /// Check out the branch belonging to the first pull request
    pub checkout: bool,
    /// Choose a github repository, using the `origin` remote of the current
    /// repository by default
    pub remote_name: Option<String>,
    /// A list of pull requests to fetch
    pub prs: Vec<Pr>,
}

impl PrFetch {
    pub const BRANCH_NAME_FLAG: CliFlag<'static> = CliFlag {
        short: "-b=",
        long: "--branch-name=",
        description: "Choose local name for the branch belonging to the preceding pull request",
    };

    pub const CHECKOUT_FLAG: CliFlag<'static> = CliFlag {
        short: "-c",
        long: "--checkout",
        description: "Check out the branch belonging to the first pull request",
    };

    pub const REPO_NAME_FLAG: CliFlag<'static> = CliFlag {
        short: "-r=",
        long: "--repo-name=",
        description: "Choose a github repository, using the `origin` remote of the current \
                      repository by default",
    };
}

impl SubCommand for PrFetch {
    const NAME: &str = "pr-fetch";

    fn parse<I: Iterator<Item = String>>(
        args: &mut I,
        global_flag: &mut HelpOrVersion,
    ) -> Result<Self, CliParseError> {
        let mut prs: Vec<Pr> = vec![];
        let mut checkout = false;
        let mut repo_name = None;

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
                },
                Some(LocalFlag::RepoName(custom_repo_name)) => {
                    if repo_name.is_some() {
                        return Err(CliParseError::DuplicateFlag(Flag::LocalFlag(
                            LocalFlag::RepoName(custom_repo_name),
                        )));
                    }
                    if custom_repo_name.is_empty() {
                        return Err(CliParseError::EmptyArgument(arg.clone()));
                    }
                    repo_name = Some(custom_repo_name);
                },
                Some(LocalFlag::BranchName(custom_branch_name)) => {
                    let Some(pr) = prs.last_mut() else {
                        return Err(CliParseError::BranchNameInvalidPosition(custom_branch_name));
                    };
                    if pr.custom_branch_name.is_some() {
                        return Err(CliParseError::DuplicateFlag(Flag::LocalFlag(
                            LocalFlag::BranchName(custom_branch_name),
                        )));
                    }
                    pr.custom_branch_name = Some(custom_branch_name);
                },
                Some(flag) => return Err(CliParseError::UnexpectedFlag(flag)),
                None => {
                    let parse_pr = |pr: &str| {
                        pr.parse::<u32>()
                            .map_err(|_err| CliParseError::InvalidArgument(pr.to_owned()))
                    };
                    let (pr_number, commit) = match arg.split_once('@') {
                        Some((pr_number, commit)) => {
                            if commit.is_empty() {
                                return Err(CliParseError::EmptyCommitHash(arg.clone()));
                            }
                            (parse_pr(pr_number)?, Some(commit))
                        },
                        None => (parse_pr(&arg)?, None),
                    };
                    let commit = commit.map(|c| Commit::parse(c.to_owned())).transpose()?;
                    prs.push(Pr {
                        number: pr_number,
                        commit,
                        custom_branch_name: None,
                    });
                },
            }
        }

        if checkout && prs.is_empty() {
            return Err(CliParseError::CheckoutNoSource);
        }

        Ok(PrFetch {
            checkout,
            remote_name: repo_name,
            prs,
        })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cli::tests::patchy;
    use crate::cli::{Cli, Subcommand};

    #[test]
    fn single_pr() {
        assert_eq!(
            patchy(&["pr-fetch", "11745"]),
            Ok(Cli {
                subcommand: Some(Subcommand::PrFetch(PrFetch {
                    checkout: false,
                    remote_name: None,
                    prs: vec![Pr {
                        number: 11745,
                        commit: None,
                        custom_branch_name: None,
                    }],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );
    }

    #[test]
    fn custom_branch_names() {
        assert_eq!(
            patchy(&[
                "pr-fetch",
                "11745",
                "10000",
                "--branch-name=some-pr",
                "9191",
                "--branch-name=another-pr",
                "600"
            ]),
            Ok(Cli {
                subcommand: Some(Subcommand::PrFetch(PrFetch {
                    checkout: false,
                    remote_name: None,
                    prs: vec![
                        Pr {
                            number: 11745,
                            commit: None,
                            custom_branch_name: None,
                        },
                        Pr {
                            number: 10000,
                            commit: None,
                            custom_branch_name: Some("some-pr".to_owned()),
                        },
                        Pr {
                            number: 9191,
                            commit: None,
                            custom_branch_name: Some("another-pr".to_owned()),
                        },
                        Pr {
                            number: 600,
                            commit: None,
                            custom_branch_name: None,
                        }
                    ],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );

        // With short flag for branch name
        assert_eq!(
            patchy(&["pr-fetch", "11745", "10000", "-b=some-pr"]),
            Ok(Cli {
                subcommand: Some(Subcommand::PrFetch(PrFetch {
                    checkout: false,
                    remote_name: None,
                    prs: vec![
                        Pr {
                            number: 11745,
                            commit: None,
                            custom_branch_name: None,
                        },
                        Pr {
                            number: 10000,
                            commit: None,
                            custom_branch_name: Some("some-pr".to_owned()),
                        }
                    ],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );
    }

    #[test]
    fn with_repo_name() {
        assert_eq!(
            patchy(&[
                "pr-fetch",
                "--repo-name=helix-editor/helix",
                "11745",
                "10000"
            ]),
            Ok(Cli {
                subcommand: Some(Subcommand::PrFetch(PrFetch {
                    checkout: false,
                    remote_name: Some("helix-editor/helix".to_owned()),
                    prs: vec![
                        Pr {
                            number: 11745,
                            commit: None,
                            custom_branch_name: None,
                        },
                        Pr {
                            number: 10000,
                            commit: None,
                            custom_branch_name: None,
                        }
                    ],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );

        assert_eq!(
            patchy(&["pr-fetch", "-r=helix-editor/helix", "11745"]),
            Ok(Cli {
                subcommand: Some(Subcommand::PrFetch(PrFetch {
                    checkout: false,
                    remote_name: Some("helix-editor/helix".to_owned()),
                    prs: vec![Pr {
                        number: 11745,
                        commit: None,
                        custom_branch_name: None,
                    }],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );
    }

    #[test]
    fn with_checkout_flag() {
        assert_eq!(
            patchy(&["pr-fetch", "11745", "--checkout"]),
            Ok(Cli {
                subcommand: Some(Subcommand::PrFetch(PrFetch {
                    checkout: true,
                    remote_name: None,
                    prs: vec![Pr {
                        number: 11745,
                        commit: None,
                        custom_branch_name: None,
                    }],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );

        assert_eq!(
            patchy(&["pr-fetch", "11745", "-c"]),
            Ok(Cli {
                subcommand: Some(Subcommand::PrFetch(PrFetch {
                    checkout: true,
                    remote_name: None,
                    prs: vec![Pr {
                        number: 11745,
                        commit: None,
                        custom_branch_name: None,
                    }],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );
    }

    #[test]
    fn at_specific_commit() {
        assert_eq!(
            patchy(&[
                "pr-fetch",
                "11745",
                "10000@be8f264327f6ae729a0b372ef01f6fde49a78310",
                "9191",
                "600@5d10fa5beb917a0dbe0ef8441d14b3d0dd15227b"
            ]),
            Ok(Cli {
                subcommand: Some(Subcommand::PrFetch(PrFetch {
                    checkout: false,
                    remote_name: None,
                    prs: vec![
                        Pr {
                            number: 11745,
                            commit: None,
                            custom_branch_name: None,
                        },
                        Pr {
                            number: 10000,
                            commit: Some(
                                Commit::parse(
                                    "be8f264327f6ae729a0b372ef01f6fde49a78310".to_owned()
                                )
                                .unwrap()
                            ),
                            custom_branch_name: None,
                        },
                        Pr {
                            number: 9191,
                            commit: None,
                            custom_branch_name: None,
                        },
                        Pr {
                            number: 600,
                            commit: Some(
                                Commit::parse(
                                    "5d10fa5beb917a0dbe0ef8441d14b3d0dd15227b".to_owned()
                                )
                                .unwrap()
                            ),
                            custom_branch_name: None,
                        }
                    ],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );
    }

    #[test]
    fn help_and_version_flags() {
        assert_eq!(
            patchy(&["pr-fetch", "--help"]),
            Ok(Cli {
                subcommand: Some(Subcommand::PrFetch(PrFetch {
                    checkout: false,
                    remote_name: None,
                    prs: vec![],
                })),
                help_or_version: HelpOrVersion::Help,
            })
        );

        assert_eq!(
            patchy(&["pr-fetch", "--version"]),
            Ok(Cli {
                subcommand: Some(Subcommand::PrFetch(PrFetch {
                    checkout: false,
                    remote_name: None,
                    prs: vec![],
                })),
                help_or_version: HelpOrVersion::Version,
            })
        );
    }

    #[test]
    fn invalid_cases() {
        assert_eq!(
            patchy(&["pr-fetch", "--yes"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::Yes))
        );
        assert_eq!(
            patchy(&["pr-fetch", "--patch-filename=test"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::PatchFilename(
                "test".to_owned()
            )))
        );
        assert_eq!(
            patchy(&["pr-fetch", "invalid-pr"]),
            Err(CliParseError::InvalidArgument("invalid-pr".to_owned()))
        );
        assert_eq!(
            patchy(&["pr-fetch", "--branch-name="]),
            Err(CliParseError::BranchNameInvalidPosition(String::new()))
        );
        assert_eq!(
            patchy(&["pr-fetch", "--repo-name="]),
            Err(CliParseError::EmptyArgument("--repo-name=".to_owned()))
        );
    }

    #[test]
    fn different_options_and_flags() {
        assert_eq!(
            patchy(&[
                "pr-fetch",
                "--repo-name=helix-editor/helix",
                "11745",
                "10000@be8f264327f6ae729a0b372ef01f6fde49a78310",
                "--branch-name=custom-branch",
                "--checkout"
            ]),
            Ok(Cli {
                subcommand: Some(Subcommand::PrFetch(PrFetch {
                    checkout: true,
                    remote_name: Some("helix-editor/helix".to_owned()),
                    prs: vec![
                        Pr {
                            number: 11745,
                            commit: None,
                            custom_branch_name: None,
                        },
                        Pr {
                            number: 10000,
                            commit: Some(
                                Commit::parse(
                                    "be8f264327f6ae729a0b372ef01f6fde49a78310".to_owned()
                                )
                                .unwrap()
                            ),
                            custom_branch_name: Some("custom-branch".to_owned()),
                        }
                    ],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );
    }

    #[test]
    fn multiple_prs() {
        assert_eq!(
            patchy(&["pr-fetch", "11745", "10000", "9191", "600"]),
            Ok(Cli {
                subcommand: Some(Subcommand::PrFetch(PrFetch {
                    checkout: false,
                    remote_name: None,
                    prs: vec![
                        Pr {
                            number: 11745,
                            commit: None,
                            custom_branch_name: None,
                        },
                        Pr {
                            number: 10000,
                            commit: None,
                            custom_branch_name: None,
                        },
                        Pr {
                            number: 9191,
                            commit: None,
                            custom_branch_name: None,
                        },
                        Pr {
                            number: 600,
                            commit: None,
                            custom_branch_name: None,
                        }
                    ],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );
    }

    #[test]
    fn pr_number_with_at_but_no_commit() {
        assert_eq!(
            patchy(&["pr-fetch", "11745@"]),
            Err(CliParseError::EmptyCommitHash("11745@".to_owned()))
        );
    }

    #[test]
    fn non_numeric_pr_number() {
        assert_eq!(
            patchy(&["pr-fetch", "abc"]),
            Err(CliParseError::InvalidArgument("abc".to_owned()))
        );
    }

    #[test]
    fn leading_zeroes() {
        assert_eq!(
            patchy(&["pr-fetch", "00123"]),
            Ok(Cli {
                subcommand: Some(Subcommand::PrFetch(PrFetch {
                    checkout: false,
                    remote_name: None,
                    prs: vec![Pr {
                        number: 123,
                        commit: None,
                        custom_branch_name: None,
                    }],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );
    }

    #[test]
    fn check_repo_name_pr() {
        assert_eq!(
            patchy(&["pr-fetch", "--checkout", "--repo-name=test", "11745"]),
            Ok(Cli {
                subcommand: Some(Subcommand::PrFetch(PrFetch {
                    checkout: true,
                    remote_name: Some("test".to_owned()),
                    prs: vec![Pr {
                        number: 11745,
                        commit: None,
                        custom_branch_name: None,
                    }],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );
    }

    #[test]
    fn pr_number_special_characters() {
        assert_eq!(
            patchy(&["pr-fetch", "123$456"]),
            Err(CliParseError::InvalidArgument("123$456".to_owned()))
        );
    }

    #[test]
    fn invalid_commit_hash() {
        assert_eq!(
            patchy(&["pr-fetch", "123@xyz!"]),
            Err(CliParseError::InvalidCommitHash("xyz!".to_owned()))
        );
    }

    #[test]
    fn flag_without_values() {
        assert_eq!(
            patchy(&["pr-fetch", "--repo-name="]),
            Err(CliParseError::EmptyArgument("--repo-name=".to_owned()))
        );

        assert_eq!(
            patchy(&["pr-fetch", "--branch-name=hello"]),
            Err(CliParseError::BranchNameInvalidPosition("hello".to_owned()))
        );
        assert_eq!(
            patchy(&["pr-fetch", "-r="]),
            Err(CliParseError::EmptyArgument("-r=".to_owned()))
        );

        assert_eq!(
            patchy(&["pr-fetch", "-b="]),
            Err(CliParseError::BranchNameInvalidPosition(String::new()))
        );
    }

    #[test]
    fn duplicate_flags() {
        assert_eq!(
            patchy(&["pr-fetch", "--checkout", "--checkout", "123"]),
            Err(CliParseError::DuplicateFlag(Flag::LocalFlag(
                LocalFlag::Checkout
            )))
        );

        assert_eq!(
            patchy(&["pr-fetch", "--repo-name=test1", "--repo-name=test2", "123"]),
            Err(CliParseError::DuplicateFlag(Flag::LocalFlag(
                LocalFlag::RepoName("test2".to_owned())
            )))
        );
    }

    #[test]
    fn checkout_with_no_source() {
        assert_eq!(
            patchy(&["pr-fetch", "-c"]),
            Err(CliParseError::CheckoutNoSource)
        );
        assert_eq!(
            patchy(&["pr-fetch", "--checkout"]),
            Err(CliParseError::CheckoutNoSource)
        );
    }

    #[test]
    fn forgot_flag_dash() {
        assert_eq!(
            patchy(&["pr-fetch", "checkout", "123"]),
            Err(CliParseError::InvalidArgument("checkout".to_owned()))
        );
    }
}
