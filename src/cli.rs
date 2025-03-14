//! Logic for parsing command line arguments
//!
//! Why not `clap`: Attempts were made to transition to `clap`, however `clap`
//! lacks features and we would have to implement basically everything from
//! scratch anyways.
//! - `clap` doesn't allow passing flags for every argument (positional flags)
//! - `clap` offers less control over how the help output is shown than we'd
//!   like, which means we would have write that from scratch if we want a good
//!   help menu.
use core::{error, fmt};
use std::env;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CliParseError {
    /// Flag exists but is not valid in this position
    UnexpectedFlag(LocalFlag),
    UnknownFlag(String),
    InvalidArgument(String),
    DuplicateFlag(Flag),
    // for now just --version and --help
    MutuallyExclusiveFlags,
    UnknownArgument(String),
    EmptyArgument(String),
}

impl fmt::Display for CliParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliParseError::UnexpectedFlag(flag) => write!(f, "Unexpected flag: {flag}"),
            CliParseError::DuplicateFlag(flag) => write!(f, "Cannot use {flag} more than once"),
            CliParseError::MutuallyExclusiveFlags => write!(
                f,
                "Flags --help and --version are mutually exclusive, so they cannot be used \
                 together."
            ),
            CliParseError::UnknownArgument(arg) => write!(f, "Unknown argument: {arg}"),
            CliParseError::EmptyArgument(arg) => write!(f, "Empty argument: {arg}"),
            CliParseError::InvalidArgument(arg) => write!(f, "Invalid argument: {arg}"),
            CliParseError::UnknownFlag(flag) => write!(f, "Unknown flag: {flag}"),
        }
    }
}

impl error::Error for CliParseError {}

/// These flags are only available when used in a specific command
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LocalFlag {
    Yes,
    Checkout,
    PatchFilename(String),
    RepoName(String),
    BranchName(String),
}

impl LocalFlag {
    /// Returns `Ok(None)`: When the argument is not a flag
    fn parse(arg: &str) -> Result<Option<Self>, CliParseError> {
        if Flag::YES_FLAGS.contains(&arg) {
            Ok(Some(LocalFlag::Yes))
        } else if Flag::CHECKOUT_FLAGS.contains(&arg) {
            Ok(Some(LocalFlag::Checkout))
        } else if let Some(value) = Flag::extract_value_flag(Flag::PATCH_FILENAME_FLAGS, arg) {
            Ok(Some(LocalFlag::PatchFilename(value.to_owned())))
        } else if let Some(value) = Flag::extract_value_flag(Flag::REPO_NAME_FLAGS, arg) {
            Ok(Some(LocalFlag::RepoName(value.to_owned())))
        } else if let Some(value) = Flag::extract_value_flag(Flag::BRANCH_NAME_FLAGS, arg) {
            Ok(Some(LocalFlag::BranchName(value.to_owned())))
        } else if arg.starts_with('-') {
            Err(CliParseError::UnknownFlag(arg.to_owned()))
        } else {
            Ok(None) // Not a flag
        }
    }
}

impl fmt::Display for LocalFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocalFlag::Yes => write!(f, "{}", Flag::YES_FLAGS[1]),
            LocalFlag::Checkout => write!(f, "{}", Flag::CHECKOUT_FLAGS[1]),
            LocalFlag::PatchFilename(name) => write!(f, "{}{name}", Flag::PATCH_FILENAME_FLAGS[1]),
            LocalFlag::RepoName(name) => write!(f, "{}{name}", Flag::REPO_NAME_FLAGS[1]),
            LocalFlag::BranchName(name) => write!(f, "{}{name}", Flag::BRANCH_NAME_FLAGS[1]),
        }
    }
}

/// These flags are available always
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GlobalFlag {
    Help,
    Version,
    #[default]
    None,
}

impl GlobalFlag {
    /// Returns:
    /// - `Ok(true)`: If flag was modified
    /// - `Ok(false)`: If flag wasn't modified, and no error was encountered
    pub fn detect(&mut self, arg: &str) -> Result<bool, CliParseError> {
        if Flag::HELP_FLAGS.contains(&arg) {
            match self {
                GlobalFlag::Version => Err(CliParseError::MutuallyExclusiveFlags),
                GlobalFlag::Help => Err(CliParseError::DuplicateFlag(Flag::GlobalFlag(
                    GlobalFlag::Help,
                ))),
                GlobalFlag::None => {
                    *self = GlobalFlag::Help;
                    Ok(true)
                },
            }
        } else if Flag::VERSION_FLAGS.contains(&arg) {
            match self {
                GlobalFlag::Version => Err(CliParseError::DuplicateFlag(Flag::GlobalFlag(
                    GlobalFlag::Version,
                ))),
                GlobalFlag::Help => Err(CliParseError::MutuallyExclusiveFlags),
                GlobalFlag::None => {
                    *self = Self::Version;
                    Ok(true)
                },
            }
        } else {
            Ok(false)
        }
    }
}

impl fmt::Display for GlobalFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlobalFlag::Help => write!(f, "{}", Flag::HELP_FLAGS[1]),
            GlobalFlag::Version => write!(f, "{}", Flag::VERSION_FLAGS[1]),
            GlobalFlag::None => write!(f, "<no flag>"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Flag {
    LocalFlag(LocalFlag),
    GlobalFlag(GlobalFlag),
}

impl Flag {
    pub const YES_FLAGS: &[&str; 2] = &["-y", "--yes"];
    pub const CHECKOUT_FLAGS: &[&str; 2] = &["-c", "--checkout"];
    pub const PATCH_FILENAME_FLAGS: &[&str; 2] = &["-n=", "--patch-filename="];
    pub const REPO_NAME_FLAGS: &[&str; 2] = &["-r=", "--repo-name="];
    pub const BRANCH_NAME_FLAGS: &[&str; 2] = &["-b=", "--branch-name="];
    pub const HELP_FLAGS: &[&str; 2] = &["-h", "--help"];
    pub const VERSION_FLAGS: &[&str; 2] = &["-v", "--version"];

    /// # Examples
    ///
    /// ```
    /// use patchy::cli::Flag;
    ///
    /// assert_eq!(
    ///     Flag::extract_value_flag(&["-m=", "--my-flag="], "-m=hello"),
    ///     Some("hello")
    /// );
    /// assert_eq!(
    ///     Flag::extract_value_flag(&["-m=", "--my-flag="], "--my-flag=hello"),
    ///     Some("hello")
    /// );
    /// assert_eq!(
    ///     Flag::extract_value_flag(&["-m=", "--my-flag="], "doesn't have"),
    ///     None
    /// );
    /// ```
    pub fn extract_value_flag<'a>(
        flags: &'static [&'static str; 2],
        arg: &'a str,
    ) -> Option<&'a str> {
        arg.strip_prefix(flags[0])
            .or_else(|| arg.strip_prefix(flags[1]))
    }
}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Flag::LocalFlag(local_flag) => write!(f, "{local_flag}"),
            Flag::GlobalFlag(global_flag) => write!(f, "{global_flag}"),
        }
    }
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

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cli {
    pub subcommand: Option<Subcommand>,
    pub global_flag: GlobalFlag,
}

impl Cli {
    /// Parse the command line arguments passed to Patchy
    pub fn parse() -> Result<Self, CliParseError> {
        Self::__parse(env::args())
    }

    /// To allow this function to be used in tests
    pub fn __parse<Args: Iterator<Item = String>>(mut args: Args) -> Result<Self, CliParseError> {
        let mut cli = Self::default();

        // skip the name used to invoke Patchy, we don't care about that
        let _ = args.next();

        while let Some(arg) = args.next() {
            if cli.global_flag.detect(&arg)? {
                continue;
            }

            match arg.as_str() {
                "init" => {
                    for arg in args.by_ref() {
                        if cli.global_flag.detect(&arg)? {
                            continue;
                        };
                        return Err((LocalFlag::parse(&arg)?)
                            .map_or(CliParseError::InvalidArgument(arg), |flag| {
                                CliParseError::UnexpectedFlag(flag)
                            }));
                    }
                    cli.subcommand = Some(Subcommand::Init);
                },
                "run" => {
                    let mut yes = false;
                    for arg in args.by_ref() {
                        if cli.global_flag.detect(&arg)? {
                            continue;
                        };
                        match LocalFlag::parse(&arg)? {
                            Some(flag @ LocalFlag::Yes) => {
                                if yes {
                                    return Err(CliParseError::DuplicateFlag(Flag::LocalFlag(
                                        flag,
                                    )));
                                }
                                yes = true;
                            },
                            Some(flag) => return Err(CliParseError::UnexpectedFlag(flag)),
                            None => return Err(CliParseError::InvalidArgument(arg)),
                        }
                    }
                    cli.subcommand = Some(Subcommand::Run { yes });
                },
                "gen-patch" => {
                    let mut patches: Vec<Patch> = vec![];

                    for arg in args.by_ref() {
                        if cli.global_flag.detect(&arg)? {
                            continue;
                        };

                        match LocalFlag::parse(&arg)? {
                            Some(LocalFlag::PatchFilename(custom_filename)) => {
                                let Some(patch) = patches.last_mut() else {
                                    return Err(CliParseError::InvalidArgument(format!(
                                        "{} must follow a commit hash",
                                        LocalFlag::PatchFilename(custom_filename)
                                    )));
                                };
                                patch.custom_filename = Some(custom_filename);
                            },
                            Some(flag) => return Err(CliParseError::UnexpectedFlag(flag)),
                            None => {
                                // TODO: validate the commit hash that it is a valid commit hash
                                // This is a commit hash
                                patches.push(Patch {
                                    commit: arg,
                                    custom_filename: None,
                                });
                            },
                        }
                    }
                    cli.subcommand = Some(Subcommand::GenPatch { patches });
                },
                "pr-fetch" => {
                    let mut prs: Vec<Pr> = vec![];
                    let mut checkout = false;
                    let mut repo_name = None;

                    for arg in args.by_ref() {
                        if cli.global_flag.detect(&arg)? {
                            continue;
                        };

                        match LocalFlag::parse(&arg)? {
                            Some(flag @ LocalFlag::Checkout) => {
                                if checkout {
                                    return Err(CliParseError::DuplicateFlag(Flag::LocalFlag(
                                        flag,
                                    )));
                                }
                                checkout = true;
                            },
                            Some(LocalFlag::RepoName(custom_repo_name)) => {
                                if repo_name.is_some() {
                                    return Err(CliParseError::DuplicateFlag(Flag::LocalFlag(
                                        LocalFlag::RepoName(custom_repo_name),
                                    )));
                                }
                                repo_name = Some(custom_repo_name);
                            },
                            Some(LocalFlag::BranchName(custom_branch_name)) => {
                                let Some(pr) = prs.last_mut() else {
                                    return Err(CliParseError::InvalidArgument(format!(
                                        "{} must follow a PR number",
                                        LocalFlag::BranchName(custom_branch_name)
                                    )));
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
                                // Parse PR number with optional commit
                                let parse_pr = |pr: &str| {
                                    pr.parse::<u32>().map_err(|_err| {
                                        CliParseError::InvalidArgument(format!(
                                            "{pr} is not a valid pull request number. Examples of \
                                             valid pull request numbers:
                                             - 41250
                                             - 1455
                                             - 10000"
                                        ))
                                    })
                                };
                                let (pr_number, commit) = match arg.split_once('@') {
                                    Some((pr_number, commit)) => {
                                        if commit.is_empty() {
                                            return Err(CliParseError::EmptyArgument(format!(
                                                "Commit cannot be empty for {arg}"
                                            )));
                                        };
                                        (parse_pr(pr_number)?, Some(commit))
                                    },
                                    None => (parse_pr(&arg)?, None),
                                };
                                prs.push(Pr {
                                    number: pr_number,
                                    commit: commit.map(ToOwned::to_owned),
                                    custom_branch_name: None,
                                });
                            },
                        }
                    }
                    cli.subcommand = Some(Subcommand::PrFetch {
                        checkout,
                        repo_name,
                        prs,
                    });
                },
                "branch-fetch" => {
                    let mut branches: Vec<Branch> = vec![];

                    for arg in args.by_ref() {
                        if cli.global_flag.detect(&arg)? {
                            continue;
                        };

                        // Non-flag arguments for branch-fetch are always branch names with optional
                        // commits
                        if LocalFlag::parse(&arg)?.is_none() {
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
                        } else {
                            // Only global flags should be parsed for branch-fetch
                            return Err(CliParseError::InvalidArgument(arg));
                        }
                    }
                    cli.subcommand = Some(Subcommand::BranchFetch { branches });
                },
                arg if arg.starts_with('-') => {
                    return Err(CliParseError::UnknownFlag(arg.to_owned()));
                },
                arg => return Err(CliParseError::UnknownArgument(arg.to_owned())),
            }
        }

        Ok(cli)
    }
}

#[cfg(test)]
mod tests {
    use core::iter;

    use pretty_assertions::assert_eq;

    use super::*;

    #[track_caller]
    fn patchy(args: &[&str]) -> Result<Cli, CliParseError> {
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
