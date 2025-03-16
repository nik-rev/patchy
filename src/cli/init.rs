use super::{CliParseError, GlobalFlag, LocalFlag, SubCommand};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Init;

impl SubCommand for Init {
    fn parse<I: Iterator<Item = String>>(
        args: &mut I,
        global_flag: &mut GlobalFlag,
    ) -> Result<Self, CliParseError> {
        for arg in args.by_ref() {
            if let Ok(flag) = arg.parse::<GlobalFlag>() {
                global_flag.validate(flag)?;
                continue;
            }

            return Err(LocalFlag::parse(&arg)?
                .map_or(CliParseError::InvalidArgument(arg), |flag| {
                    CliParseError::UnexpectedFlag(flag)
                }));
        }

        Ok(Init)
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
            patchy(&["init"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init(Init)),
                global_flag: GlobalFlag::None,
            })
        );
        assert_eq!(
            patchy(&["init", "--help"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init(Init)),
                global_flag: GlobalFlag::Help,
            })
        );
        assert_eq!(
            patchy(&["init", "-h"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init(Init)),
                global_flag: GlobalFlag::Help,
            })
        );
        assert_eq!(
            patchy(&["--help", "init"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init(Init)),
                global_flag: GlobalFlag::Help,
            })
        );
        assert_eq!(
            patchy(&["-h", "init"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init(Init)),
                global_flag: GlobalFlag::Help,
            })
        );
        assert_eq!(
            patchy(&["--version", "init"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init(Init)),
                global_flag: GlobalFlag::Version,
            })
        );
        assert_eq!(
            patchy(&["-v", "init"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init(Init)),
                global_flag: GlobalFlag::Version,
            })
        );
        assert_eq!(
            patchy(&["init", "--version"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init(Init)),
                global_flag: GlobalFlag::Version,
            })
        );
        assert_eq!(
            patchy(&["init", "-v"]),
            Ok(Cli {
                subcommand: Some(Subcommand::Init(Init)),
                global_flag: GlobalFlag::Version,
            })
        );
    }

    #[test]
    fn invalid() {
        assert_eq!(
            patchy(&["init", "--yes"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::Yes))
        );
        assert_eq!(
            patchy(&["init", "-y"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::Yes))
        );
        assert_eq!(
            patchy(&["init", "hello"]),
            Err(CliParseError::InvalidArgument("hello".to_owned()))
        );
        assert_eq!(
            patchy(&["init", "--patch-filename=test"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::PatchFilename(
                "test".to_owned()
            )))
        );
        assert_eq!(
            patchy(&["init", "--branch-name=test"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::BranchName(
                "test".to_owned()
            )))
        );
        assert_eq!(
            patchy(&["init", "--checkout"]),
            Err(CliParseError::UnexpectedFlag(LocalFlag::Checkout))
        );
    }
}
