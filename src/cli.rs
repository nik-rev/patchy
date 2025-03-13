//! Logic for parsing command line arguments
//!
//! Why not `clap`: Attempts were made to transition to `clap`, however `clap` lacks features and we would have to implement basically everything from scratch anyways.
//! - `clap` doesn't allow passing flags for every argument (positional flags)
//! - `clap` offers less control over how the help output is shown than we'd like, which means we would have write that from scratch if we want a good help menu.
use core::{error, fmt};
use std::{collections::HashSet, env, fmt::Display, str::FromStr};

use non_empty_string::NonEmptyString;

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cli {
    pub subcommand: Option<Subcommand>,
    pub help: bool,
    pub version: bool,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParseError {
    NoSubcommandSpecified,
    InvalidFlag(GlobalFlag),
    InvalidArg(String),
    InvalidCommandName(String),
    MutuallyExclusiveFlags(GlobalFlag, GlobalFlag),
    DuplicateFlag(GlobalFlag),
    EmptyFilename,
    FlagMustFollow(String, String),
    UnknownFlag(String),
    NoArguments,
    Todo,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSubcommandSpecified => write!(f, "No command name"),
            Self::InvalidCommandName(name) => write!(f, "Invalid command: {name}"),
            Self::MutuallyExclusiveFlags(one, two) => {
                write!(f, "{one} and {two} cannot be used together")
            }
            Self::DuplicateFlag(flag) => write!(f, "{flag} can only be used once"),
            Self::InvalidFlag(flag) => write!(f, "Invalid flag: {flag}"),
            Self::InvalidArg(flag) => write!(f, "Invalid argument: {flag}"),
            Self::EmptyFilename => todo!(),
            Self::FlagMustFollow(arg, flag) => write!(f, "{flag} must follow {arg}"),
            Self::UnknownFlag(flag) => write!(f, "Unknown flag: {flag}"),
            Self::NoArguments => write!(f, "No arguments were supplied"),
            Self::Todo => write!(f, "todo"),
        }
    }
}

impl error::Error for ParseError {}

/// These flags can only be used once
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum GlobalFlag {
    Help,
    Version,
    Yes,
}

impl GlobalFlag {
    /// The set of flags which this flag cannot be used with
    fn conflicts_with(self) -> HashSet<Self> {
        match self {
            Self::Help => HashSet::from([Self::Version]),
            Self::Version => HashSet::from([Self::Help]),
            Self::Yes => HashSet::new(),
        }
    }
}

impl FromStr for GlobalFlag {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-h" | "--help" => Ok(Self::Help),
            "-v" | "--version" => Ok(Self::Version),
            "-y" | "--yes" => Ok(Self::Yes),
            _ => Err(()),
        }
    }
}

impl Display for GlobalFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Help => write!(f, "--help"),
            Self::Version => write!(f, "--version"),
            Self::Yes => write!(f, "--yes"),
        }
    }
}

impl Cli {
    pub fn parse() -> Result<Self, ParseError> {
        Self::__parse(env::args())
    }

    /// To allow this function to be used in tests
    pub fn __parse<Args: Iterator<Item = String>>(mut args: Args) -> Result<Self, ParseError> {
        let mut cli = Self::default();

        // skip the name used to invoke Patchy, we don't care about that
        let _ = args.next();

        let args: Vec<_> = args.collect();

        let mut global_flags = HashSet::<GlobalFlag>::new();
        let mut other_args = vec![];

        // PERF: Could be improved if we don't iterate twice over args
        // Though in reality this should not be a problem
        for arg in args {
            if let Ok(flag) = arg.parse::<GlobalFlag>() {
                if global_flags.contains(&flag) {
                    return Err(ParseError::DuplicateFlag(flag));
                };
                if let Some(conflict) = global_flags.intersection(&flag.conflicts_with()).next() {
                    return Err(ParseError::MutuallyExclusiveFlags(*conflict, flag));
                };
                global_flags.insert(flag);
            } else {
                // not a flag
                other_args.push(arg);
            };
        }

        if global_flags.remove(&GlobalFlag::Help) {
            cli.help = true;
        }

        if global_flags.remove(&GlobalFlag::Version) {
            cli.version = true;
        }

        let mut args = other_args.into_iter();

        let Some(command_name) = args.next() else {
            // When no command name is supplied, we can only supply GlobalFlag::Version or GlobalFlag::Help
            // Both of which have been removed earlier. So we should have *no* global flags now.
            if let Some(flag) = global_flags.into_iter().next() {
                return Err(ParseError::InvalidFlag(flag));
            }

            // Literally nothing was supplied
            if !cli.help && !cli.version {
                return Err(ParseError::NoSubcommandSpecified);
            }
            return Ok(cli);
        };

        match command_name.as_str() {
            "init" => {
                cli.subcommand = Some(Subcommand::Init);
                // takes no flags
                if let Some(flag) = global_flags.into_iter().next() {
                    return Err(ParseError::InvalidFlag(flag));
                }
                // takes no arguments
                if let Some(arg) = args.next() {
                    return Err(ParseError::InvalidArg(arg));
                }
            }
            "run" => {
                let mut yes = false;
                if global_flags.remove(&GlobalFlag::Yes) {
                    yes = true;
                };
                // takes no other flags
                if let Some(flag) = global_flags.into_iter().next() {
                    return Err(ParseError::InvalidFlag(flag));
                }
                // takes no arguments
                if let Some(arg) = args.next() {
                    return Err(ParseError::InvalidArg(arg));
                }
                cli.subcommand = Some(Subcommand::Run { yes });
            }
            "gen-patch" => {
                let patches = args.try_fold(vec![], |mut patches: Vec<Patch>, arg| {
                    if let Some(custom_filename) = arg
                        .strip_prefix("-n=")
                        .or_else(|| arg.strip_prefix("--patch-filename="))
                    {
                        if custom_filename.is_empty() {
                            return Err(ParseError::EmptyFilename);
                        }

                        let Some(last_patch) = patches.last_mut() else {
                            return Err(ParseError::FlagMustFollow(
                                "commit".to_owned(),
                                "--patch-filename=".to_owned(),
                            ));
                        };

                        if last_patch.custom_filename.is_some() {
                            // "--path-filename has been used before"
                            return Err(ParseError::Todo);
                        }

                        last_patch.custom_filename = Some(custom_filename.to_owned());

                        Ok(patches)
                    } else if arg.starts_with('-') {
                        Err(ParseError::UnknownFlag(arg))
                    } else {
                        patches.push(Patch {
                            commit: arg,
                            custom_filename: None,
                        });

                        Ok(patches)
                    }
                })?;

                if patches.is_empty() {
                    return Err(ParseError::NoArguments);
                }

                cli.subcommand = Some(Subcommand::GenPatch { patches });
            }
            // "pr-fetch" => {
            //     for arg in args {
            //         health_and_version(&arg)?;
            //         todo!()
            //         // command.parse_arg(arg);
            //     }
            // }
            // "branch-fetch" => {
            //     for arg in args {
            //         health_and_version(&arg)?;
            //         todo!()
            //         // command.parse_arg(arg);
            //     }
            // }
            _ => return Err(ParseError::InvalidCommandName(command_name)),
        }

        Ok(cli)
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
    /// When fetching this PR, rename the branch fetched to this string
    custom_branch_name: Option<String>,
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

    #[test]
    fn empty() {
        assert_eq!(patchy(&[]), Err(ParseError::NoSubcommandSpecified));
    }

    #[test]
    fn run() {
        assert_eq!(
            patchy(&["run"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run { yes: false }),
                help: false,
                version: false
            })
        );
        assert_eq!(
            patchy(&["run", "--yes"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run { yes: true }),
                help: false,
                version: false
            })
        );
        assert_eq!(
            patchy(&["-y", "run"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run { yes: true }),
                help: false,
                version: false
            })
        );
        assert_eq!(
            patchy(&["-h", "run"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run { yes: false }),
                help: true,
                version: false
            })
        );
        assert_eq!(
            patchy(&["-v", "run"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run { yes: false }),
                help: false,
                version: true
            })
        );
        assert_eq!(
            patchy(&["--yes", "run", "--help"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run { yes: true }),
                help: true,
                version: false
            })
        );
    }

    #[test]
    fn init() {
        assert_eq!(
            patchy(&["init"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init),
                help: false,
                version: false
            })
        );
        assert_eq!(
            patchy(&["init", "--help"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init),
                help: true,
                version: false
            })
        );
        assert_eq!(
            patchy(&["--version", "init"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init),
                help: false,
                version: true
            })
        );
        assert_eq!(
            patchy(&["init", "--version"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init),
                help: false,
                version: true
            })
        );
        assert_eq!(
            patchy(&["init", "-h"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init),
                help: true,
                version: false
            })
        );
        assert_eq!(
            patchy(&["init", "--yes"]),
            Err(ParseError::InvalidFlag(GlobalFlag::Yes))
        );
        assert_eq!(
            patchy(&["init", "hello"]),
            Err(ParseError::InvalidArg("hello".to_owned()))
        );
    }

    #[test]
    fn no_command() {
        assert_eq!(patchy(&[]), Err(ParseError::NoSubcommandSpecified));
        assert_eq!(
            patchy(&["-h"]),
            Ok(Cli {
                subcommand: None,
                help: true,
                version: false
            })
        );
        assert_eq!(
            patchy(&["--help"]),
            Ok(Cli {
                subcommand: None,
                help: true,
                version: false
            })
        );
        assert_eq!(
            patchy(&["-v"]),
            Ok(Cli {
                subcommand: None,
                help: false,
                version: true
            })
        );
        assert_eq!(
            patchy(&["--version"]),
            Ok(Cli {
                subcommand: None,
                help: false,
                version: true
            })
        );
    }

    #[test]
    fn no_command_mutually_exclusive_flag() {
        assert_eq!(
            patchy(&["-h", "--version"]),
            Err(ParseError::MutuallyExclusiveFlags(
                GlobalFlag::Help,
                GlobalFlag::Version
            ))
        );
        assert_eq!(
            patchy(&["--help", "--version"]),
            Err(ParseError::MutuallyExclusiveFlags(
                GlobalFlag::Help,
                GlobalFlag::Version
            ))
        );
        assert_eq!(
            patchy(&["-v", "--help"]),
            Err(ParseError::MutuallyExclusiveFlags(
                GlobalFlag::Version,
                GlobalFlag::Help
            ))
        );
        assert_eq!(
            patchy(&["-v", "init", "-h"]),
            Err(ParseError::MutuallyExclusiveFlags(
                GlobalFlag::Version,
                GlobalFlag::Help
            ))
        );
        assert_eq!(
            patchy(&["-v", "-h", "init", "-h"]),
            Err(ParseError::MutuallyExclusiveFlags(
                GlobalFlag::Version,
                GlobalFlag::Help
            ))
        );
        assert_eq!(
            patchy(&["-v", "-h", "init", "-v"]),
            Err(ParseError::MutuallyExclusiveFlags(
                GlobalFlag::Version,
                GlobalFlag::Help
            ))
        );
    }

    #[test]
    fn no_command_duplicate_flag() {
        assert_eq!(
            patchy(&["-h", "--help"]),
            Err(ParseError::DuplicateFlag(GlobalFlag::Help))
        );
        assert_eq!(
            patchy(&["-v", "--version"]),
            Err(ParseError::DuplicateFlag(GlobalFlag::Version))
        );
    }

    #[test]
    fn gen_patch_no_arg() {
        assert_eq!(patchy(&["gen-patch"]), Err(ParseError::NoArguments));
    }

    #[test]
    fn gen_patch_extra_flag() {
        assert_eq!(
            patchy(&[
                "gen-patch",
                "133cbaae83f710b7",
                "--patch-filename=hi",
                "--patch-filename=bye"
            ]),
            Err(ParseError::Todo)
        );
    }

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
                help: false,
                version: false
            })
        );
    }

    #[test]
    fn gen_patch_1_arg() {
        assert_eq!(
            patchy(&["gen-patch", "133cbaae83f710b7", "--patch-filename="]),
            Err(ParseError::EmptyFilename)
        );
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
                help: false,
                version: false
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
                help: true,
                version: false
            })
        );
        assert_eq!(
            patchy(&["gen-patch", "133cbaae83f710b7", "-n="]),
            Err(ParseError::EmptyFilename)
        );
        assert_eq!(
            patchy(&["gen-patch", "133cbaae83f710b7", "-n=helloworld"]),
            Ok(Cli {
                subcommand: Some(Subcommand::GenPatch {
                    patches: vec![Patch {
                        commit: "133cbaae83f710b7".to_owned(),
                        custom_filename: Some("helloworld".to_owned())
                    }]
                }),
                help: false,
                version: false
            })
        );
    }
}
