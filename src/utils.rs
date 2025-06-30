//! Utilities for patchy

use std::{fmt::Display, sync::LazyLock};

use anyhow::anyhow;
use colored::Colorize as _;
use rand::{Rng as _, distributions};
use reqwest::{Client, header::USER_AGENT};
use tap::Pipe as _;

use crate::config::PrNumber;

/// Add a uuid identifier to the string to make it unique
pub fn with_uuid(s: &str) -> String {
    let uuid = rand::thread_rng()
        .sample_iter(&distributions::Alphanumeric)
        .take(4)
        .map(char::from)
        .collect::<String>();

    format!("{uuid}-{s}",)
}

/// Converts a commit message to only contain lowercase characters,
/// underscores and dashes
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

/// Format the pull request for display in the terminal
pub fn format_pr(pr: PrNumber, pr_title: &str, url: &str) -> String {
    format_url(
        format!(
            "{}{}{}{}",
            "#".bright_blue(),
            pr.to_string().bright_blue(),
            " ".bright_blue(),
            pr_title.bright_blue().italic()
        ),
        url,
    )
}

/// Style a snippet of text as a link
pub fn format_url(text: impl Display, url: impl Display) -> String {
    format!("\u{1b}]8;;{url}\u{1b}\\{text}\u{1b}]8;;\u{1b}\\")
}

/// Send a GET request to the specified URL
///
/// Return the result as text
pub async fn make_request(url: &str) -> anyhow::Result<String> {
    static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);
    let request = CLIENT.get(url).header(USER_AGENT, "patchy").send().await;

    match request {
        Ok(res) if res.status().is_success() => res.text().await?.pipe(Ok),
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

/// Get a yes or no answer from the user
#[macro_export]
macro_rules! confirm_prompt {
    ($($arg:tt)*) => {{
        dialoguer::Confirm::new()
            .with_prompt(format!(
                "\n  {} {}",
                colored::Colorize::bright_black("»"),
                format!($($arg)*)
            ))
            .interact()
            .unwrap()
    }};
}
