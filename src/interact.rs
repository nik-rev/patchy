//! Utilities for user interaction

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
