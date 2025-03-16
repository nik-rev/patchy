use super::{CliParseError, Flag, GlobalFlag, LocalFlag, SubCommand};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Run {
    pub yes: bool,
}

impl SubCommand for Run {
    fn parse<I: Iterator<Item = String>>(
        args: &mut I,
        global_flag: &mut GlobalFlag,
    ) -> Result<Self, CliParseError> {
        let mut yes = false;

        for arg in args.by_ref() {
            if let Ok(flag) = arg.parse::<GlobalFlag>() {
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
    use crate::cli::{Cli, Subcommand, patchy};

    #[test]
    fn valid() {
        assert_eq!(
            patchy(&["run"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: false })),
                global_flag: GlobalFlag::None,
            })
        );
        assert_eq!(
            patchy(&["run", "--help"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: false })),
                global_flag: GlobalFlag::Help,
            })
        );
        assert_eq!(
            patchy(&["run", "-h"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: false })),
                global_flag: GlobalFlag::Help,
            })
        );
        assert_eq!(
            patchy(&["run", "--version"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: false })),
                global_flag: GlobalFlag::Version,
            })
        );
        assert_eq!(
            patchy(&["run", "-v"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: false })),
                global_flag: GlobalFlag::Version,
            })
        );
        assert_eq!(
            patchy(&["run", "--yes"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: true })),
                global_flag: GlobalFlag::None,
            })
        );
        assert_eq!(
            patchy(&["run", "-y"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: true })),
                global_flag: GlobalFlag::None,
            })
        );
        assert_eq!(
            patchy(&["--help", "run"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: false })),
                global_flag: GlobalFlag::Help,
            })
        );
        assert_eq!(
            patchy(&["--version", "run"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Run(Run { yes: false })),
                global_flag: GlobalFlag::Version,
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
