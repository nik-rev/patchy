//! Utilities for user interaction

/// Print a success message
#[macro_export]
macro_rules! note {
    ($($arg:tt)*) => {{
        println!("  {}{}",
            colored::Colorize::bold(colored::Colorize::bright_blue("note: ")),
            format!($($arg)*))
    }};
}

/// Print a success message
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        println!("  {}{}",
            colored::Colorize::bold(colored::Colorize::bright_green("✓ ")),
            format!($($arg)*))
    }};
}

/// Print a fail message
#[macro_export]
macro_rules! fail {
    ($($arg:tt)*) => {{
        eprintln!("  {}{}",
            colored::Colorize::bold(colored::Colorize::bright_red("✗ ")),
            format!($($arg)*))
    }};
}

/// Get a yes or no answer from the user
#[macro_export]
macro_rules! confirm_prompt {
    ($($arg:tt)*) => {{
        dialoguer::Confirm::new()
            .with_prompt(format!(
                "\n  {} {}",
                "»".bright_black(),
                format!($($arg)*)
            ))
            .interact()
            .unwrap()
    }};
}
