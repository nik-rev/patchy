use core::{error, fmt};
use std::env;

use flags::{Flag, GlobalFlag, LocalFlag};

mod branch_fetch;
mod flags;
mod gen_patch;
mod init;
mod pr_fetch;
mod run;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CliParseError {
    UnexpectedFlag(LocalFlag),
    UnknownFlag(String),
    InvalidArgument(String),
    InvalidRepo(String),
    DuplicateFlag(Flag),
    MutuallyExclusiveFlags,
    UnknownArgument(String),
    UnknownSubcommand(String),
    EmptyArgument(String),
    EmptyCommitHash(String),
    PatchFilenameInvalidPosition(String),
    BranchNameInvalidPosition(String),
}

impl fmt::Display for CliParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliParseError::UnexpectedFlag(flag) => write!(f, "Unexpected flag: {flag}"),
            CliParseError::DuplicateFlag(flag) => write!(f, "Cannot use {flag} more than once"),
            CliParseError::MutuallyExclusiveFlags => write!(
                f,
                "Flags {} and {} are mutually exclusive, so they cannot be used together.",
                GlobalFlag::Help,
                GlobalFlag::Version
            ),
            CliParseError::UnknownArgument(arg) => write!(f, "Unknown argument: {arg}"),
            CliParseError::EmptyArgument(arg) => write!(f, "Empty argument: {arg}"),
            CliParseError::InvalidArgument(arg) => write!(f, "Invalid argument: {arg}"),
            CliParseError::UnknownFlag(flag) => write!(f, "Unknown flag: {flag}"),
            CliParseError::UnknownSubcommand(subcommand) => {
                write!(f, "Unknown subcommand: {subcommand}")
            },
            CliParseError::PatchFilenameInvalidPosition(filename) => {
                write!(
                    f,
                    "{} must follow a commit hash",
                    LocalFlag::PatchFilename(filename.to_string())
                )
            },
            CliParseError::BranchNameInvalidPosition(name) => {
                write!(
                    f,
                    "{} must follow a pull request number",
                    LocalFlag::PatchFilename(name.to_string())
                )
            },
            CliParseError::EmptyCommitHash(pr) => {
                write!(f, "{pr} must be followed by a commit hash")
            },
            CliParseError::InvalidRepo(repo) => write!(f, "Invalid repo: {repo}"),
        }
    }
}

impl error::Error for CliParseError {}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Subcommand {
    Init(init::Init),
    Run(run::Run),
    GenPatch(gen_patch::GenPatch),
    PrFetch(pr_fetch::PrFetch),
    BranchFetch(branch_fetch::BranchFetch),
}

trait SubCommand {
    fn parse<I: Iterator<Item = String>>(
        args: &mut I,
        global_flag: &mut GlobalFlag,
    ) -> Result<Self, CliParseError>
    where
        Self: Sized;
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
        // skip the name used to invoke Patchy, we don't care about that
        let _ = args.next();

        let mut global_flag = GlobalFlag::None;
        let mut subcommand = None;

        // Process global flags before the subcommand
        let mut arg_queue = Vec::new();

        for arg in args.by_ref() {
            if let Ok(flag) = arg.parse::<GlobalFlag>() {
                global_flag.validate(flag)?;
            } else if flags::is_flag(&arg) {
                // only expect global flags until this point
                return Err(CliParseError::UnknownFlag(arg));
            } else {
                arg_queue.push(arg);
                break;
            }
        }

        if let Some(cmd) = arg_queue.pop() {
            subcommand = Some(match cmd.as_str() {
                "init" => Subcommand::Init(init::Init::parse(&mut args, &mut global_flag)?),
                "run" => Subcommand::Run(run::Run::parse(&mut args, &mut global_flag)?),
                "gen-patch" => {
                    Subcommand::GenPatch(gen_patch::GenPatch::parse(&mut args, &mut global_flag)?)
                },
                "pr-fetch" => {
                    Subcommand::PrFetch(pr_fetch::PrFetch::parse(&mut args, &mut global_flag)?)
                },
                "branch-fetch" => Subcommand::BranchFetch(branch_fetch::BranchFetch::parse(
                    &mut args,
                    &mut global_flag,
                )?),
                arg => return Err(CliParseError::UnknownSubcommand(arg.to_owned())),
            });
        }

        Ok(Cli {
            subcommand,
            global_flag,
        })
    }
}

/// Calls `patchy` with the given command line arguments
#[cfg(test)]
#[track_caller]
fn patchy(args: &[&str]) -> Result<Cli, CliParseError> {
    dbg!(args);
    Cli::__parse(std::iter::once("patchy".to_owned()).chain(args.iter().map(ToString::to_string)))
}

#[cfg(test)]
mod tests {
    use gen_patch::Patch;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn global_flags() {
        assert_eq!(
            patchy(&["--help"]),
            Ok(Cli {
                subcommand: None,
                global_flag: GlobalFlag::Help,
            })
        );
        assert_eq!(
            patchy(&["-h"]),
            Ok(Cli {
                subcommand: None,
                global_flag: GlobalFlag::Help,
            })
        );
        assert_eq!(
            patchy(&["--version"]),
            Ok(Cli {
                subcommand: None,
                global_flag: GlobalFlag::Version,
            })
        );
        assert_eq!(
            patchy(&["-v"]),
            Ok(Cli {
                subcommand: None,
                global_flag: GlobalFlag::Version,
            })
        );
    }

    #[test]
    fn invalid_global_flags() {
        assert_eq!(
            patchy(&["--unknown-flag"]),
            Err(CliParseError::UnknownFlag("--unknown-flag".to_owned()))
        );
        assert_eq!(
            patchy(&["-u"]),
            Err(CliParseError::UnknownFlag("-u".to_owned()))
        );
    }

    #[test]
    fn mutually_exclusive_global_flags() {
        assert_eq!(
            patchy(&["--help", "--version"]),
            Err(CliParseError::MutuallyExclusiveFlags)
        );
        assert_eq!(
            patchy(&["-h", "-v"]),
            Err(CliParseError::MutuallyExclusiveFlags)
        );
    }

    #[test]
    fn unknown_subcommand() {
        assert_eq!(
            patchy(&["unknown-command"]),
            Err(CliParseError::UnknownSubcommand(
                "unknown-command".to_owned()
            ))
        );
        assert_eq!(
            patchy(&["--help", "unknown-command"]),
            Err(CliParseError::UnknownSubcommand(
                "unknown-command".to_owned()
            ))
        );
    }

    #[test]
    fn no_arguments() {
        assert_eq!(
            patchy(&[]),
            Ok(Cli {
                subcommand: None,
                global_flag: GlobalFlag::None,
            })
        );
    }
}
