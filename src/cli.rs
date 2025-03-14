//! Logic for parsing command line arguments
//!
//! Why not `clap`: Attempts were made to transition to `clap`, however `clap` lacks features and we would have to implement basically everything from scratch anyways.
//! - `clap` doesn't allow passing flags for every argument (positional flags)
//! - `clap` offers less control over how the help output is shown than we'd like, which means we would have write that from scratch if we want a good help menu.
use core::{error, fmt};
use std::env;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParseError {
    UnknownFlag(String),
    InvalidArgument(String),
    DuplicateFlag(String),
    // for now just --version and --help
    MutuallyExclusiveFlags,
    UnknownArgument(String),
    EmptyArgument(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnknownFlag(_) => todo!(),
            ParseError::DuplicateFlag(_) => todo!(),
            ParseError::MutuallyExclusiveFlags => todo!(),
            ParseError::UnknownArgument(_) => todo!(),
            ParseError::EmptyArgument(_) => todo!(),
            ParseError::InvalidArgument(_) => todo!(),
        }
    }
}

impl error::Error for ParseError {}

impl Cli {
    pub fn parse() -> Result<Self, ParseError> {
        Self::__parse(env::args())
    }

    /// To allow this function to be used in tests
    pub fn __parse<Args: Iterator<Item = String>>(mut args: Args) -> Result<Self, ParseError> {
        let mut cli = Self::default();

        // skip the name used to invoke Patchy, we don't care about that
        let _ = args.next();

        macro_rules! version_and_help {
            ($arg:ident) => {
                if $arg == "-h" || $arg == "--help" {
                    match cli.global_flag {
                        GlobalFlag::Version => return Err(ParseError::MutuallyExclusiveFlags),
                        GlobalFlag::Help => {
                            return Err(ParseError::DuplicateFlag("--help".to_string()));
                        }
                        GlobalFlag::None => {
                            cli.global_flag = GlobalFlag::Help;
                            continue;
                        }
                    }
                }
                if $arg == "-v" || $arg == "--version" {
                    match cli.global_flag {
                        GlobalFlag::Version => {
                            return Err(ParseError::DuplicateFlag("--version".to_string()));
                        }
                        GlobalFlag::Help => return Err(ParseError::MutuallyExclusiveFlags),
                        GlobalFlag::None => {
                            cli.global_flag = GlobalFlag::Version;
                            continue;
                        }
                    }
                }
            };
        }

        #[expect(clippy::while_let_on_iterator, reason = "todo")]
        while let Some(arg) = args.next() {
            version_and_help!(arg);

            match arg.as_str() {
                "init" => {
                    while let Some(arg) = args.next() {
                        version_and_help!(arg);
                        return Err(ParseError::InvalidArgument(arg));
                    }
                    cli.subcommand = Some(Subcommand::Init);
                }
                "run" => {
                    let mut yes = false;
                    while let Some(arg) = args.next() {
                        version_and_help!(arg);
                        if arg == "-y" || arg == "--yes" {
                            if yes {
                                return Err(ParseError::DuplicateFlag("--yes".to_owned()));
                            }
                            yes = true;
                        } else if arg.starts_with('-') {
                            return Err(ParseError::UnknownFlag(arg));
                        } else {
                            return Err(ParseError::InvalidArgument(arg));
                        }
                    }
                    cli.subcommand = Some(Subcommand::Run { yes });
                }
                "gen-patch" => {
                    let mut patches: Vec<Patch> = vec![];
                    while let Some(arg) = args.next() {
                        version_and_help!(arg);
                        if let Some(custom_filename) = arg
                            .strip_prefix("--patch-filename=")
                            .or_else(|| arg.strip_prefix("-n="))
                        {
                            let Some(patch) = patches.last_mut() else {
                                return Err(ParseError::InvalidArgument(
                                    "--patch-filename= must follow a commit hash".to_owned(),
                                ));
                            };
                            patch.custom_filename = Some(custom_filename.to_owned());
                        } else if arg.starts_with('-') {
                            return Err(ParseError::UnknownFlag(arg));
                        } else {
                            // TODO: validate the commit
                            patches.push(Patch {
                                commit: arg,
                                custom_filename: None,
                            });
                        }
                    }
                    cli.subcommand = Some(Subcommand::GenPatch { patches });
                }
                "pr-fetch" => {
                    let mut prs: Vec<Pr> = vec![];
                    let mut checkout = false;
                    let mut repo_name = None;
                    while let Some(arg) = args.next() {
                        version_and_help!(arg);
                        if arg == "-c" || arg == "--checkout" {
                            if checkout {
                                return Err(ParseError::DuplicateFlag("--checkout".to_owned()));
                            }
                            checkout = true;
                        } else if let Some(custom_repo_name) = arg
                            .strip_prefix("--repo-name=")
                            .or_else(|| arg.strip_prefix("-r="))
                        {
                            if repo_name.is_some() {
                                return Err(ParseError::DuplicateFlag("--repo-name=".to_owned()));
                            }
                            repo_name = Some(custom_repo_name.to_owned());
                        } else if let Some(custom_branch_name) = arg
                            .strip_prefix("--branch-name=")
                            .or_else(|| arg.strip_prefix("-b="))
                        {
                            let Some(pr) = prs.last_mut() else {
                                return Err(ParseError::InvalidArgument(
                                    "--branch-name must follow a PR number".to_owned(),
                                ));
                            };
                            pr.custom_branch_name = Some(custom_branch_name.to_owned());
                        } else if arg.starts_with('-') {
                            return Err(ParseError::UnknownFlag(arg));
                        } else {
                            let (pr_number, commit) = match arg.split_once('@') {
                                Some((pr_number, commit)) => {
                                    if commit.is_empty() {
                                        return Err(ParseError::EmptyArgument(format!(
                                            "commit is empty for {arg}"
                                        )));
                                    };
                                    let pr_number = pr_number.parse::<u32>().map_err(|_err| {
                                        ParseError::InvalidArgument(format!(
                                            "{pr_number} is not a valid pull request number"
                                        ))
                                    })?;
                                    (pr_number, Some(commit))
                                }
                                None => (
                                    arg.parse::<u32>().map_err(|_err| {
                                        ParseError::InvalidArgument(format!(
                                            "{arg} is not a valid pull request number"
                                        ))
                                    })?,
                                    None,
                                ),
                            };
                            prs.push(Pr {
                                number: pr_number,
                                commit: commit.map(ToOwned::to_owned),
                                custom_branch_name: None,
                            });
                        }
                    }
                    cli.subcommand = Some(Subcommand::PrFetch {
                        checkout,
                        repo_name,
                        prs,
                    });
                }
                "branch-fetch" => {
                    let mut branches: Vec<Branch> = vec![];
                    while let Some(arg) = args.next() {
                        version_and_help!(arg);

                        let (branch_name, commit) = match arg.split_once('@') {
                            Some((branch_name, commit)) => {
                                if commit.is_empty() {
                                    return Err(ParseError::EmptyArgument(format!(
                                        "commit is empty for {arg}"
                                    )));
                                };
                                (branch_name, Some(commit))
                            }
                            None => (arg.as_str(), None),
                        };

                        branches.push(Branch {
                            name: branch_name.to_owned(),
                            commit: commit.map(ToOwned::to_owned),
                        });
                    }
                    cli.subcommand = Some(Subcommand::BranchFetch { branches });
                }
                arg if arg.starts_with('-') => return Err(ParseError::UnknownFlag(arg.to_owned())),
                arg => return Err(ParseError::UnknownArgument(arg.to_owned())),
            }
        }

        Ok(cli)
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cli {
    pub subcommand: Option<Subcommand>,
    pub global_flag: GlobalFlag,
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GlobalFlag {
    Help,
    Version,
    #[default]
    None,
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Patch {
    pub commit: String,
    pub custom_filename: Option<String>,
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pr {
    /// Fetch PR of this number
    number: u32,
    /// When fetching this PR, reset to this commit
    commit: Option<String>,
    /// When fetching this PR, rename the branch fetched to this string
    custom_branch_name: Option<String>,
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Branch {
    /// Name of this branch in the remote
    name: String,
    /// When fetching this PR, reset to this commit
    commit: Option<String>,
    // When fetching this PR, rename the branch fetched to this string
    // custom_branch_name: Option<String>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Subcommand {
    Init,
    Run {
        /// If true, do not prompt for user confirmation when overwriting
        yes: bool,
    },
    GenPatch {
        /// A list of patches to apply
        patches: Vec<Patch>,
    },
    PrFetch {
        /// `git checkout` the first fetched pull request
        checkout: bool,
        repo_name: Option<String>,
        prs: Vec<Pr>,
    },
    BranchFetch {
        branches: Vec<Branch>,
    },
}

#[cfg(test)]
mod tests {
    use core::iter;
    use pretty_assertions::assert_eq;

    use super::*;

    #[track_caller]
    fn patchy(args: &[&str]) -> Result<Cli, ParseError> {
        dbg!(args);
        Cli::__parse(iter::once("patchy".to_owned()).chain(args.iter().map(ToString::to_string)))
    }

    // #[test]
    // fn empty() {
    //     assert_eq!(patchy(&[]), Err(ParseError::NoSubcommandSpecified));
    // }

    #[test]
    fn run() {
        assert_eq!(
            patchy(&["run"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run { yes: false }),
                global_flag: GlobalFlag::None,
            })
        );
        assert_eq!(
            patchy(&["run", "--yes"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run { yes: true }),
                global_flag: GlobalFlag::None,
            })
        );
        assert_eq!(
            patchy(&["-h", "run"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run { yes: false }),
                global_flag: GlobalFlag::Help,
            })
        );
        assert_eq!(
            patchy(&["-v", "run"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run { yes: false }),
                global_flag: GlobalFlag::Version,
            })
        );
    }

    #[test]
    fn init() {
        assert_eq!(
            patchy(&["init"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init),
                global_flag: GlobalFlag::None,
            })
        );
        assert_eq!(
            patchy(&["init", "--help"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init),
                global_flag: GlobalFlag::Help,
            })
        );
        assert_eq!(
            patchy(&["--version", "init"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init),
                global_flag: GlobalFlag::Version,
            })
        );
        assert_eq!(
            patchy(&["init", "--version"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init),
                global_flag: GlobalFlag::Version,
            })
        );
        assert_eq!(
            patchy(&["init", "-h"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init),
                global_flag: GlobalFlag::Help,
            })
        );
        // assert_eq!(
        //     patchy(&["init", "--yes"]),
        //     Err(ParseError::UnknownFlag(GlobalFlag::Yes))
        // );
        // assert_eq!(
        //     patchy(&["init", "hello"]),
        //     Err(ParseError::InvalidArg("hello".to_owned()))
        // );
    }

    #[test]
    fn no_command() {
        // assert_eq!(patchy(&[]), Err(ParseError::NoSubcommandSpecified));
        assert_eq!(
            patchy(&["-h"]),
            Ok(Cli {
                subcommand: None,
                global_flag: GlobalFlag::Help,
            })
        );
        assert_eq!(
            patchy(&["--help"]),
            Ok(Cli {
                subcommand: None,
                global_flag: GlobalFlag::Help,
            })
        );
        assert_eq!(
            patchy(&["-v"]),
            Ok(Cli {
                subcommand: None,
                global_flag: GlobalFlag::Version,
            })
        );
        assert_eq!(
            patchy(&["--version"]),
            Ok(Cli {
                subcommand: None,
                global_flag: GlobalFlag::Version,
            })
        );
    }

    // #[test]
    // fn no_command_mutually_exclusive_flag() {
    //     assert_eq!(
    //         patchy(&["-h", "--version"]),
    //         Err(ParseError::MutuallyExclusiveFlags(
    //             GlobalFlag::Help,
    //             GlobalFlag::Version
    //         ))
    //     );
    //     assert_eq!(
    //         patchy(&["--help", "--version"]),
    //         Err(ParseError::MutuallyExclusiveFlags(
    //             GlobalFlag::Help,
    //             GlobalFlag::Version
    //         ))
    //     );
    //     assert_eq!(
    //         patchy(&["-v", "--help"]),
    //         Err(ParseError::MutuallyExclusiveFlags(
    //             GlobalFlag::Version,
    //             GlobalFlag::Help
    //         ))
    //     );
    //     assert_eq!(
    //         patchy(&["-v", "init", "-h"]),
    //         Err(ParseError::MutuallyExclusiveFlags(
    //             GlobalFlag::Version,
    //             GlobalFlag::Help
    //         ))
    //     );
    //     assert_eq!(
    //         patchy(&["-v", "-h", "init", "-h"]),
    //         Err(ParseError::MutuallyExclusiveFlags(
    //             GlobalFlag::Version,
    //             GlobalFlag::Help
    //         ))
    //     );
    //     assert_eq!(
    //         patchy(&["-v", "-h", "init", "-v"]),
    //         Err(ParseError::MutuallyExclusiveFlags(
    //             GlobalFlag::Version,
    //             GlobalFlag::Help
    //         ))
    //     );
    // }

    // #[test]
    // fn no_command_duplicate_flag() {
    //     assert_eq!(
    //         patchy(&["-h", "--help"]),
    //         Err(ParseError::DuplicateFlag(GlobalFlag::Help))
    //     );
    //     assert_eq!(
    //         patchy(&["-v", "--version"]),
    //         Err(ParseError::DuplicateFlag(GlobalFlag::Version))
    //     );
    // }

    // #[test]
    // fn gen_patch_no_arg() {
    //     assert_eq!(patchy(&["gen-patch"]), Err(ParseError::NoArguments));
    // }

    // #[test]
    // fn gen_patch_extra_flag() {
    //     assert_eq!(
    //         patchy(&[
    //             "gen-patch",
    //             "133cbaae83f710b7",
    //             "--patch-filename=hi",
    //             "--patch-filename=bye"
    //         ]),
    //         Err(ParseError::Todo)
    //     );
    // }

    #[test]
    fn gen_patch_multiple_flags() {
        assert_eq!(
            patchy(&[
                "gen-patch",
                "133cbaae83f710b793c98018cea697a04479bbe4",
                "--patch-filename=some-patch",
                "9ad5aa637ccf363b5d6713f66d0c2830736c35a9",
                "--patch-filename=another-patch",
                "cc75a895f344cf2fe83eaf6d78dfb7aeac8b33a4",
            ]),
            Ok(Cli {
                subcommand: Some(Subcommand::GenPatch {
                    patches: vec![
                        Patch {
                            commit: "133cbaae83f710b793c98018cea697a04479bbe4".to_owned(),
                            custom_filename: Some("some-patch".to_owned())
                        },
                        Patch {
                            commit: "9ad5aa637ccf363b5d6713f66d0c2830736c35a9".to_owned(),
                            custom_filename: Some("another-patch".to_owned())
                        },
                        Patch {
                            commit: "cc75a895f344cf2fe83eaf6d78dfb7aeac8b33a4".to_owned(),
                            custom_filename: None
                        }
                    ]
                }),
                global_flag: GlobalFlag::None,
            })
        );
    }

    #[test]
    fn gen_patch_1_arg() {
        // assert_eq!(
        //     patchy(&["gen-patch", "133cbaae83f710b7", "--patch-filename="]),
        //     Err(ParseError::EmptyFilename)
        // );
        assert_eq!(
            patchy(&[
                "gen-patch",
                "133cbaae83f710b7",
                "--patch-filename=hello-world"
            ]),
            Ok(Cli {
                subcommand: Some(Subcommand::GenPatch {
                    patches: vec![Patch {
                        commit: "133cbaae83f710b7".to_owned(),
                        custom_filename: Some("hello-world".to_owned())
                    }]
                }),
                global_flag: GlobalFlag::None,
            })
        );
        assert_eq!(
            patchy(&[
                "gen-patch",
                "133cbaae83f710b7",
                "--patch-filename=hello-world",
                "--help"
            ]),
            Ok(Cli {
                subcommand: Some(Subcommand::GenPatch {
                    patches: vec![Patch {
                        commit: "133cbaae83f710b7".to_owned(),
                        custom_filename: Some("hello-world".to_owned())
                    }]
                }),
                global_flag: GlobalFlag::Help,
            })
        );
        // assert_eq!(
        //     patchy(&["gen-patch", "133cbaae83f710b7", "-n="]),
        //     Err(ParseError::EmptyFilename)
        // );
        assert_eq!(
            patchy(&["gen-patch", "133cbaae83f710b7", "-n=helloworld"]),
            Ok(Cli {
                subcommand: Some(Subcommand::GenPatch {
                    patches: vec![Patch {
                        commit: "133cbaae83f710b7".to_owned(),
                        custom_filename: Some("helloworld".to_owned())
                    }]
                }),
                global_flag: GlobalFlag::None,
            })
        );
    }
}
