use std::str::FromStr;

use colored::Colorize as _;
use documented::DocumentedVariants;

use super::branch_fetch::BranchFetch;
use super::gen_patch::GenPatch;
use super::pr_fetch::PrFetch;
use super::run::Run;
use super::{Cli, CliParseError, fmt};

pub struct CliFlag<'a> {
    pub short: &'a str,
    pub long: &'a str,
    pub description: &'a str,
}

impl CliFlag<'_> {
    fn is(&self, s: &str) -> bool {
        self.short == s || self.long == s
    }

    /// Extract the value part of a flag argument if it matches one of the
    /// provided flags
    pub fn extract_value_flag<'a>(&self, arg: &'a str) -> Option<&'a str> {
        arg.strip_prefix(self.short)
            .or_else(|| arg.strip_prefix(self.long))
    }
}

impl fmt::Display for CliFlag<'_> {
    /// Formats a flag into a colored format with a description, printable to
    /// the terminal
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}\n    {}",
            self.short.bright_magenta(),
            ", ".bright_black(),
            self.long.bright_magenta(),
            crate::commands::help::format_description(self.description)
        )
    }
}

/// Extract the value part of a flag argument if it matches one of the
/// provided flags
pub fn extract_value_flag<'a>(flags: &'static [&'static str; 2], arg: &'a str) -> Option<&'a str> {
    arg.strip_prefix(flags[0])
        .or_else(|| arg.strip_prefix(flags[1]))
}

/// Check if an argument is a flag
pub fn is_flag(arg: &str) -> bool {
    arg.starts_with('-')
}

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
    pub fn parse(arg: &str) -> Result<Option<Self>, CliParseError> {
        if Run::YES_FLAG.is(arg) {
            Ok(Some(LocalFlag::Yes))
        } else if BranchFetch::CHECKOUT_FLAG.is(arg) {
            Ok(Some(LocalFlag::Checkout))
        } else if let Some(value) = GenPatch::PATCH_NAME_FLAG.extract_value_flag(arg) {
            Ok(Some(LocalFlag::PatchFilename(value.to_owned())))
        } else if let Some(value) = PrFetch::REPO_NAME_FLAG.extract_value_flag(arg) {
            Ok(Some(LocalFlag::RepoName(value.to_owned())))
        } else if let Some(value) = BranchFetch::BRANCH_NAME_FLAG.extract_value_flag(arg) {
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
            LocalFlag::Yes => write!(f, "{}", Run::YES_FLAG.long),
            LocalFlag::Checkout => write!(f, "{}", BranchFetch::CHECKOUT_FLAG.long),
            LocalFlag::PatchFilename(name) => write!(f, "{}{name}", GenPatch::PATCH_NAME_FLAG.long),
            LocalFlag::RepoName(name) => write!(f, "{}{name}", PrFetch::REPO_NAME_FLAG.long),
            LocalFlag::BranchName(name) => {
                write!(f, "{}{name}", BranchFetch::BRANCH_NAME_FLAG.long)
            }
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, DocumentedVariants)]
pub enum HelpOrVersion {
    /// HELP flag
    Help,
    /// VERSION flag
    Version,
    /// NONE
    #[default]
    None,
}

impl HelpOrVersion {
    /// Validate a global flag and make sure that it doesn't conflict with
    /// existing global flags
    pub fn validate(&mut self, new_flag: HelpOrVersion) -> Result<(), CliParseError> {
        match (&self, new_flag) {
            // No existing flag, take the new one
            (HelpOrVersion::None, flag) => {
                *self = flag;
                Ok(())
            }

            // Same flag already set
            (HelpOrVersion::Help, HelpOrVersion::Help) => Err(CliParseError::DuplicateFlag(
                Flag::GlobalFlag(HelpOrVersion::Help),
            )),
            (HelpOrVersion::Version, HelpOrVersion::Version) => Err(CliParseError::DuplicateFlag(
                Flag::GlobalFlag(HelpOrVersion::Version),
            )),

            // Conflicting flags
            (HelpOrVersion::Help, HelpOrVersion::Version)
            | (HelpOrVersion::Version, HelpOrVersion::Help) => {
                Err(CliParseError::MutuallyExclusiveFlags)
            }

            // Second case is GlobalFlag::None, which shouldn't happen
            _ => {
                *self = new_flag;
                Ok(())
            }
        }
    }
}

impl FromStr for HelpOrVersion {
    type Err = ();

    fn from_str(arg: &str) -> Result<Self, Self::Err> {
        if Cli::HELP_FLAG.is(arg) {
            Ok(HelpOrVersion::Help)
        } else if Cli::VERSION_FLAG.is(arg) {
            Ok(HelpOrVersion::Version)
        } else {
            Err(())
        }
    }
}

impl fmt::Display for HelpOrVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HelpOrVersion::Help => write!(f, "{}", Cli::HELP_FLAG.long),
            HelpOrVersion::Version => write!(f, "{}", Cli::VERSION_FLAG.long),
            HelpOrVersion::None => write!(f, "<no flag>"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Flag {
    LocalFlag(LocalFlag),
    GlobalFlag(HelpOrVersion),
}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Flag::LocalFlag(local_flag) => write!(f, "{local_flag}"),
            Flag::GlobalFlag(global_flag) => write!(f, "{global_flag}"),
        }
    }
}
