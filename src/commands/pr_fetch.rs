use anyhow::anyhow;
use colored::Colorize as _;

use crate::cli::pr_fetch::{Pr, PrFetch};
use crate::git_commands::{GIT, GITHUB_REMOTE_PREFIX, GITHUB_REMOTE_SUFFIX, fetch_pull_request};
use crate::utils::display_link;
use crate::{fail, success};

/// Allow users to prefix their PRs with octothorpe, e.g. #12345 instead of
/// 12345. This is just a QOL addition since some people may use it due to habit
pub fn ignore_octothorpe(arg: &str) -> String {
    if arg.starts_with('#') {
        arg.get(1..).unwrap_or_default()
    } else {
        arg
    }
    .into()
}

pub async fn pr_fetch(mut args: PrFetch) -> anyhow::Result<()> {
    // The user hasn't provided a custom remote, so we're going to try `origin`
    // TODO: use methods on Option instead of mutating this variable
    if args.remote_name.is_none() {
        let remote = GIT(&["remote", "get-url", "origin"])?;
        if remote.starts_with(GITHUB_REMOTE_PREFIX) && remote.ends_with(GITHUB_REMOTE_SUFFIX) {
            let start = GITHUB_REMOTE_PREFIX.len();
            let end = remote.len() - GITHUB_REMOTE_SUFFIX.len();
            args.remote_name = remote.get(start..end).map(Into::into);
        };
    }
    let remote_name = args
        .remote_name
        .ok_or_else(|| anyhow!("Could not get the remote name!"))?;

    for (
        i,
        Pr {
            number: pull_request_number,
            commit,
            custom_branch_name,
        },
    ) in args.prs.iter().enumerate()
    {
        match fetch_pull_request(
            &remote_name,
            // TODO: make fetch_pull_request accept a u32 instead
            &pull_request_number.to_string(),
            custom_branch_name.as_deref(),
            commit.as_ref(),
        )
        .await
        {
            Ok((response, info)) => {
                success!(
                    "Fetched pull request {} available at branch {}{}",
                    display_link(
                        &format!(
                            "{}{}{}{}",
                            "#".bright_blue(),
                            pull_request_number.to_string().bright_blue(),
                            " ".bright_blue(),
                            response.title.bright_blue().italic()
                        ),
                        &response.html_url
                    ),
                    info.branch.local_branch_name.bright_cyan(),
                    commit
                        .clone()
                        .map(|commit_hash| format!(
                            ", at commit {}",
                            commit_hash.as_ref().bright_yellow()
                        ))
                        .unwrap_or_default()
                );

                // Attempt to cleanup after ourselves
                let _ = GIT(&["remote", "remove", &info.remote.local_remote_alias]);

                // If user uses --checkout flag, we're going to checkout the first PR only
                if i == 0 && args.checkout {
                    if let Err(cant_checkout) = GIT(&["checkout", &info.branch.local_branch_name]) {
                        fail!(
                            "Could not check out branch {}:\n{cant_checkout}",
                            info.branch.local_branch_name
                        );
                    } else {
                        success!(
                            "Automatically checked out the first branch: {}",
                            info.branch.local_branch_name
                        );
                    }
                }
            },
            Err(err) => {
                fail!("{err}");
                continue;
            },
        };
    }

    Ok(())
}
