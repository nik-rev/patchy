use anyhow::anyhow;
use rand::{Rng as _, distributions};
use reqwest::header::USER_AGENT;

use crate::git_commands::CLIENT;

pub fn with_uuid(s: &str) -> String {
    format!(
        "{uuid}-{s}",
        uuid = rand::thread_rng()
            .sample_iter(&distributions::Alphanumeric)
            .take(4)
            .map(char::from)
            .collect::<String>()
    )
}

/// Converts a commit message to only contain lowercase characters, underscores
/// and dashes
pub fn normalize_commit_msg(commit_msg: &str) -> String {
    commit_msg
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() {
                '_'
            } else {
                '-'
            }
        })
        .collect()
}

pub fn display_link(text: &str, url: &str) -> String {
    format!("\u{1b}]8;;{url}\u{1b}\\{text}\u{1b}]8;;\u{1b}\\")
}

pub async fn make_request(url: &str) -> anyhow::Result<String> {
    let request = CLIENT
        .get(url)
        .header(USER_AGENT, "{APP_NAME}")
        .send()
        .await;

    match request {
        Ok(res) if res.status().is_success() => {
            let out = res.text().await?;

            Ok(out)
        }
        Ok(res) => {
            let status = res.status();
            let text = res.text().await?;

            Err(anyhow!(
                "Request failed with status: {status}\nRequested URL: {url}\nResponse: {text}",
            ))
        }
        Err(err) => Err(anyhow!("Error sending request: {err}")),
    }
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        println!("  {}{}",
            colored::Colorize::bold(colored::Colorize::bright_green("✓ ")),
            format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! fail {
    ($($arg:tt)*) => {{
        eprintln!("  {}{}",
            colored::Colorize::bold(colored::Colorize::bright_red("✗ ")),
            format!($($arg)*))
    }};
}

/// Interact with the user to get a yes or a no answer
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
