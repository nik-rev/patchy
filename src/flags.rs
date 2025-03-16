use core::fmt;
use core::fmt::Display;
use std::env;

use colored::Colorize as _;
use once_cell::sync::Lazy;

use crate::commands::help::format_description;
use crate::types::CommandArgs;

pub struct Flag<'a> {
    pub short: &'a str,
    pub long: &'a str,
    pub description: &'a str,
}

/// Extracts value out of a `flag` which can have an assignment
///
/// # Examples
///
/// ```rust
/// use patchy::flags::Flag;
///
/// let my_flag = Flag {
///     short: "-r=",
///     long: "--remote-name=",
///     description: "some flag",
/// };
///
/// let long_version = my_flag.extract_from_arg("--remote-name=abc");
/// let short_version = my_flag.extract_from_arg("-r=abcdefg");
/// let invalid = my_flag.extract_from_arg("-m=abcdefg");
///
/// assert_eq!(long_version, Some("abc".into()));
/// assert_eq!(short_version, Some("abcdefg".into()));
/// assert_eq!(invalid, None);
/// ```
impl Flag<'_> {
    pub fn is_in(&self, args: &CommandArgs) -> bool {
        args.contains(self.short) || args.contains(self.long)
    }

    pub fn extract_from_arg(&self, arg: &str) -> Option<String> {
        if arg.starts_with(self.short) {
            arg.get(self.short.len()..).map(Into::into)
        } else if arg.starts_with(self.long) {
            arg.get(self.long.len()..).map(Into::into)
        } else {
            None
        }
    }
}

impl Display for Flag<'_> {
    /// Formats a flag into a colored format with a description, printable to
    /// the terminal
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}\n    {}",
            self.short.bright_magenta(),
            ", ".bright_black(),
            self.long.bright_magenta(),
            format_description(self.description)
        )
    }
}

/// Checks whether an input argument is a valid flag
pub fn is_valid_flag(arg: &str, available_flags: &[&Flag]) -> bool {
    // TODO: flags that don't end in "=" should be compared fully, not just the
    // beginning
    available_flags
        .iter()
        .flat_map(|flag| [flag.short, flag.long])
        .any(|flag| arg.starts_with(flag))
}

/// Makes the program output more detailed information
pub static IS_VERBOSE: Lazy<bool> = Lazy::new(|| {
    let args: CommandArgs = env::args().collect();
    args.contains("--verbose")
});
