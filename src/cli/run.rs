use documented::{Documented, DocumentedFields};

use super::flags::CliFlag;
use super::{CliParseError, Flag, HelpOrVersion, LocalFlag, SubCommand};

/// Start patchy
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Documented, DocumentedFields)]
pub struct Run {
    /// Do not prompt when overwriting local-branch specified in the config
    pub yes: bool,
}

impl Run {
    pub const YES_FLAG: CliFlag<'static> = CliFlag {
        short: "-y",
        long: "--yes",
        description: "Do not prompt when overwriting local-branch specified in the config",
    };
}

impl SubCommand for Run {
    const NAME: &str = "run";

    fn parse<I: Iterator<Item = String>>(
        args: &mut I,
        global_flag: &mut HelpOrVersion,
    ) -> Result<Self, CliParseError> {
        let mut yes = false;

        for arg in args.by_ref() {
            if let Ok(flag) = arg.parse::<HelpOrVersion>() {
                global_flag.validate(flag)?;
                continue;
            }

            match LocalFlag::parse(&arg)? {
                Some(flag @ LocalFlag::Yes) => {
                    if yes {
                        return Err(CliParseError::DuplicateFlag(Flag::LocalFlag(flag)));
                    }
                    yes = true;
                },
                Some(flag) => return Err(CliParseError::UnexpectedFlag(flag)),
                None => return Err(CliParseError::InvalidArgument(arg)),
            }
        }

        Ok(Run { yes })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cli::tests::patchy;
    use crate::cli::{Cli, Subcommand};

    #[test]
    fn valid() {
        assert_eq!(
            patchy(&["run"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: false })),
                help_or_version: HelpOrVersion::None,
            })
        );
        assert_eq!(
            patchy(&["run", "--help"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: false })),
                help_or_version: HelpOrVersion::Help,
            })
        );
        assert_eq!(
            patchy(&["run", "-h"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: false })),
                help_or_version: HelpOrVersion::Help,
            })
        );
        assert_eq!(
            patchy(&["run", "--version"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: false })),
                help_or_version: HelpOrVersion::Version,
            })
        );
        assert_eq!(
            patchy(&["run", "-v"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: false })),
                help_or_version: HelpOrVersion::Version,
            })
        );
        assert_eq!(
            patchy(&["run", "--yes"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: true })),
                help_or_version: HelpOrVersion::None,
            })
        );
        assert_eq!(
            patchy(&["run", "-y"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: true })),
                help_or_version: HelpOrVersion::None,
            })
        );
        assert_eq!(
            patchy(&["--help", "run"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: false })),
                help_or_version: HelpOrVersion::Help,
            })
        );
        assert_eq!(
            patchy(&["--version", "run"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: false })),
                help_or_version: HelpOrVersion::Version,
            })
        );
    }

    #[test]
    fn invalid() {
        assert_eq!(
            patchy(&["run", "hello"]),
            Err(CliParseError::InvalidArgument("hello".to_owned()))
        );
        assert_eq!(
            patchy(&["run", "--patch-filename=test"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::PatchFilename(
                "test".to_owned()
            )))
        );
        assert_eq!(
            patchy(&["run", "--branch-name=test"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::BranchName(
                "test".to_owned()
            )))
        );
        assert_eq!(
            patchy(&["run", "--checkout"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::Checkout))
        );
        assert_eq!(
            patchy(&["run", "--repo-name=test"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::RepoName(
                "test".to_owned()
            )))
        );
    }
}
