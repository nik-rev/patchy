//! Logic for parsing command line arguments
//!
//! Why not `clap`: Attempts were made to transition to `clap`, however `clap` lacks features and we would have to implement basically everything from scratch anyways.
//! - `clap` doesn't allow passing flags for every argument (positional flags)
//! - `clap` offers less control over how the help output is shown than we'd like, which means we would have write that from scratch if we want a good help menu.
use core::{error, fmt};
use std::env;

#[derive(Default)]
pub struct Cli {
    pub subcommand: Option<Subcommand>,
    pub help: bool,
    pub version: bool,
}

#[derive(Debug)]
pub enum ParseError {
    NoCommandName,
    InvalidCommandName(String),
    MutuallyExclusive(String, String),
    DuplicateFlag(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoCommandName => write!(f, "No command name"),
            Self::InvalidCommandName(name) => write!(f, "Invalid command: {name}"),
            Self::MutuallyExclusive(one, two) => {
                write!(f, "{one} and {two} cannot be used together")
            }
            Self::DuplicateFlag(flag) => write!(f, "{flag} can only be used once"),
        }
    }
}

impl error::Error for ParseError {}

impl Cli {
    pub fn parse<I: Iterator<Item = String>>(mut args: I) -> Result<Self, ParseError> {
        let mut cli = Self::default();

        // skip the name used to invoke Patchy, we don't care about that
        let _ = args.next();
        let command_name = args.next().ok_or(ParseError::NoCommandName)?;

        let args: Vec<_> = args.collect();

        let mut health_and_version = |arg: &str| {
            #[expect(clippy::useless_let_if_seq, reason = "More readable this way")]
            let mut has_found_flag = false;
            if arg == "-v" || arg == "--version" {
                // cannot set again
                if cli.version {
                    return Err(ParseError::DuplicateFlag("--version".to_owned()));
                }
                // cannot have --help and --version
                if cli.help {
                    return Err(ParseError::MutuallyExclusive(
                        "--help".to_owned(),
                        "--version".to_owned(),
                    ));
                }
                cli.version = true;
                has_found_flag = true;
            }
            if arg == "-h" || arg == "--help" {
                // cannot set again
                if cli.help {
                    return Err(ParseError::DuplicateFlag("--help".to_owned()));
                }
                // cannot have --help and --version
                if cli.version {
                    return Err(ParseError::MutuallyExclusive(
                        "--help".to_owned(),
                        "--version".to_owned(),
                    ));
                }
                cli.help = true;
                has_found_flag = true;
            }

            Ok(has_found_flag)
        };

        match command_name.as_str() {
            "init" => {
                for arg in args {
                    health_and_version(&arg)?;
                }
                cli.subcommand = Some(Subcommand::Init);
            }
            "run" => {
                let mut yes = false;
                for arg in args {
                    if health_and_version(&arg)? {
                        continue;
                    };
                    if arg == "-y" || arg == "--yes" {
                        if yes {
                            return Err(ParseError::DuplicateFlag("--yes".to_owned()));
                        }
                        yes = true;
                    }
                }
                cli.subcommand = Some(Subcommand::Run { yes });
            }
            "gen-patch" => {
                let mut patches: Vec<Patch> = vec![];
                for arg in args {
                    health_and_version(&arg)?;
                    let last = patches.last();
                }
            }
            "pr-fetch" => {
                for arg in args {
                    health_and_version(&arg)?;
                    todo!()
                    // command.parse_arg(arg);
                }
            }
            "branch-fetch" => {
                for arg in args {
                    health_and_version(&arg)?;
                    todo!()
                    // command.parse_arg(arg);
                }
            }
            _ => return Err(ParseError::InvalidCommandName(command_name)),
        }

        Ok(cli)
    }
}

#[derive(Default)]
pub struct Patch {
    commit: String,
    custom_filename: Option<String>,
}

#[derive(Default)]
pub struct Pr {
    /// Fetch PR of this number
    number: u32,
    /// When fetching this PR, reset to this commit
    commit: Option<String>,
    /// When fetching this PR, rename the branch fetched to this string
    custom_branch_name: Option<String>,
}

#[derive(Default)]
pub struct Branch {
    /// Name of this branch in the remote
    name: String,
    /// When fetching this PR, reset to this commit
    commit: Option<String>,
    /// When fetching this PR, rename the branch fetched to this string
    custom_branch_name: Option<String>,
}

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

impl Subcommand {
    pub fn parse_arg(&mut self, arg: String) {
        todo!()
    }
}
