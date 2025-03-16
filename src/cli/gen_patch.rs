use super::{CliParseError, HelpOrVersion, LocalFlag, SubCommand};

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Patch {
    pub commit: String,
    pub custom_filename: Option<String>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GenPatch {
    pub patches: Vec<Patch>,
}

impl SubCommand for GenPatch {
    fn parse<I: Iterator<Item = String>>(
        args: &mut I,
        global_flag: &mut HelpOrVersion,
    ) -> Result<Self, CliParseError> {
        let mut patches: Vec<Patch> = vec![];

        for arg in args.by_ref() {
            if let Ok(flag) = arg.parse::<HelpOrVersion>() {
                global_flag.validate(flag)?;
                continue;
            }

            match LocalFlag::parse(&arg)? {
                Some(LocalFlag::PatchFilename(custom_filename)) => {
                    let Some(patch) = patches.last_mut() else {
                        return Err(CliParseError::PatchFilenameInvalidPosition(custom_filename));
                    };
                    patch.custom_filename = Some(custom_filename);
                },
                Some(flag) => return Err(CliParseError::UnexpectedFlag(flag)),
                None => {
                    // TODO: validate the commit hash that it is a valid commit hash
                    patches.push(Patch {
                        commit: arg,
                        custom_filename: None,
                    });
                },
            }
        }

        Ok(GenPatch { patches })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cli::tests::patchy;
    use crate::cli::{Cli, Subcommand};

    const COMMIT_1: &str = "133cbaae83f710b793c98018cea697a04479bbe4";
    const COMMIT_2: &str = "9ad5aa637ccf363b5d6713f66d0c2830736c35a9";
    const COMMIT_3: &str = "cc75a895f344cf2fe83eaf6d78dfb7aeac8b33a4";

    #[test]
    fn single_commit() {
        assert_eq!(
            patchy(&["gen-patch", COMMIT_1]),
            Ok(Cli {
                subcommand: Some(Subcommand::GenPatch(GenPatch {
                    patches: vec![Patch {
                        commit: COMMIT_1.to_owned(),
                        custom_filename: None,
                    }],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );
    }
    #[test]
    fn multiple_commits() {
        assert_eq!(
            patchy(&["gen-patch", COMMIT_1, COMMIT_2, COMMIT_3]),
            Ok(Cli {
                subcommand: Some(Subcommand::GenPatch(GenPatch {
                    patches: vec![
                        Patch {
                            commit: COMMIT_1.to_owned(),
                            custom_filename: None,
                        },
                        Patch {
                            commit: COMMIT_2.to_owned(),
                            custom_filename: None,
                        },
                        Patch {
                            commit: COMMIT_3.to_owned(),
                            custom_filename: None,
                        }
                    ],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );
    }

    #[test]
    fn with_custom_filenames() {
        assert_eq!(
            patchy(&[
                "gen-patch",
                COMMIT_1,
                "--patch-filename=some-patch",
                COMMIT_2,
                "--patch-filename=another-patch",
                COMMIT_3
            ]),
            Ok(Cli {
                subcommand: Some(Subcommand::GenPatch(GenPatch {
                    patches: vec![
                        Patch {
                            commit: COMMIT_1.to_owned(),
                            custom_filename: Some("some-patch".to_owned()),
                        },
                        Patch {
                            commit: COMMIT_2.to_owned(),
                            custom_filename: Some("another-patch".to_owned()),
                        },
                        Patch {
                            commit: COMMIT_3.to_owned(),
                            custom_filename: None,
                        }
                    ],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );
        assert_eq!(
            patchy(&["gen-patch", COMMIT_1, "-n=some-patch", COMMIT_2]),
            Ok(Cli {
                subcommand: Some(Subcommand::GenPatch(GenPatch {
                    patches: vec![
                        Patch {
                            commit: COMMIT_1.to_owned(),
                            custom_filename: Some("some-patch".to_owned()),
                        },
                        Patch {
                            commit: COMMIT_2.to_owned(),
                            custom_filename: None,
                        }
                    ],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );
    }

    #[test]
    fn with_help_and_version() {
        assert_eq!(
            patchy(&["gen-patch", "--help"]),
            Ok(Cli {
                subcommand: Some(Subcommand::GenPatch(GenPatch { patches: vec![] })),
                help_or_version: HelpOrVersion::Help,
            })
        );

        assert_eq!(
            patchy(&["gen-patch", "--version"]),
            Ok(Cli {
                subcommand: Some(Subcommand::GenPatch(GenPatch { patches: vec![] })),
                help_or_version: HelpOrVersion::Version,
            })
        );
    }

    #[test]
    fn invalid() {
        assert_eq!(
            patchy(&["gen-patch", "--yes"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::Yes))
        );
        assert_eq!(
            patchy(&["gen-patch", "--checkout"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::Checkout))
        );
        assert_eq!(
            patchy(&["gen-patch", "--branch-name=test"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::BranchName(
                "test".to_owned()
            )))
        );
        assert_eq!(
            patchy(&["gen-patch", "--repo-name=test"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::RepoName(
                "test".to_owned()
            )))
        );
        assert_eq!(
            patchy(&["gen-patch", "--patch-filename="]),
            Err(CliParseError::PatchFilenameInvalidPosition(String::new()))
        );
    }

    #[test]
    fn flag_without_value() {
        assert_eq!(
            patchy(&["gen-patch", "--patch-filename="]),
            Err(CliParseError::PatchFilenameInvalidPosition(String::new()))
        );
        assert_eq!(
            patchy(&["gen-patch", "-n="]),
            Err(CliParseError::PatchFilenameInvalidPosition(String::new()))
        );
    }

    #[test]
    fn positional_flag_between_commits() {
        assert_eq!(
            patchy(&["gen-patch", "commit1", "--patch-filename=test", "commit2"]),
            Ok(Cli {
                subcommand: Some(Subcommand::GenPatch(GenPatch {
                    patches: vec![
                        Patch {
                            commit: "commit1".to_owned(),
                            custom_filename: Some("test".to_owned()),
                        },
                        Patch {
                            commit: "commit2".to_owned(),
                            custom_filename: None,
                        }
                    ],
                })),
                help_or_version: HelpOrVersion::None,
            })
        );
    }
}
