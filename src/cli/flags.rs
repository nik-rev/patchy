use std::str::FromStr;

use super::{CliParseError, flags, fmt};

pub const YES_FLAGS: &[&str; 2] = &["-y", "--yes"];
pub const CHECKOUT_FLAGS: &[&str; 2] = &["-c", "--checkout"];
pub const PATCH_FILENAME_FLAGS: &[&str; 2] = &["-n=", "--patch-filename="];
pub const REPO_NAME_FLAGS: &[&str; 2] = &["-r=", "--repo-name="];
pub const BRANCH_NAME_FLAGS: &[&str; 2] = &["-b=", "--branch-name="];
pub const HELP_FLAGS: &[&str; 2] = &["-h", "--help"];
pub const VERSION_FLAGS: &[&str; 2] = &["-v", "--version"];

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
        if flags::YES_FLAGS.contains(&arg) {
            Ok(Some(LocalFlag::Yes))
        } else if flags::CHECKOUT_FLAGS.contains(&arg) {
            Ok(Some(LocalFlag::Checkout))
        } else if let Some(value) = flags::extract_value_flag(flags::PATCH_FILENAME_FLAGS, arg) {
            Ok(Some(LocalFlag::PatchFilename(value.to_owned())))
        } else if let Some(value) = flags::extract_value_flag(flags::REPO_NAME_FLAGS, arg) {
            Ok(Some(LocalFlag::RepoName(value.to_owned())))
        } else if let Some(value) = flags::extract_value_flag(flags::BRANCH_NAME_FLAGS, arg) {
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
            LocalFlag::Yes => write!(f, "{}", flags::YES_FLAGS[1]),
            LocalFlag::Checkout => write!(f, "{}", flags::CHECKOUT_FLAGS[1]),
            LocalFlag::PatchFilename(name) => write!(f, "{}{name}", flags::PATCH_FILENAME_FLAGS[1]),
            LocalFlag::RepoName(name) => write!(f, "{}{name}", flags::REPO_NAME_FLAGS[1]),
            LocalFlag::BranchName(name) => write!(f, "{}{name}", flags::BRANCH_NAME_FLAGS[1]),
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum HelpOrVersion {
    Help,
    Version,
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
            },

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
            },

            // Second case is GlobalFlag::None, which shouldn't happen
            _ => {
                *self = new_flag;
                Ok(())
            },
        }
    }
}

impl FromStr for HelpOrVersion {
    type Err = ();

    fn from_str(arg: &str) -> Result<Self, Self::Err> {
        if flags::HELP_FLAGS.contains(&arg) {
            Ok(HelpOrVersion::Help)
        } else if flags::VERSION_FLAGS.contains(&arg) {
            Ok(HelpOrVersion::Version)
        } else {
            Err(())
        }
    }
}

impl fmt::Display for HelpOrVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HelpOrVersion::Help => write!(f, "{}", flags::HELP_FLAGS[1]),
            HelpOrVersion::Version => write!(f, "{}", flags::VERSION_FLAGS[1]),
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
