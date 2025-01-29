use crate::commands::help;
use crate::fail;
use crate::flags::{is_valid_flag, Flag};
use crate::git_commands::{
    fetch_pull_request, is_valid_branch_name, GIT, GITHUB_REMOTE_PREFIX, GITHUB_REMOTE_SUFFIX,
};
use crate::success;
use crate::types::CommandArgs;
use crate::utils::display_link;
use anyhow::anyhow;
use colored::Colorize;

use super::help::{HELP_FLAG, VERSION_FLAG};
use super::run::parse_if_maybe_hash;

/// Allow users to prefix their PRs with octothorpe, e.g. #12345 instead of 12345.
/// This is just a QOL addition since some people may use it due to habit
pub fn ignore_octothorpe(arg: &str) -> String {
    if arg.starts_with("#") {
        arg.get(1..).unwrap_or_default()
    } else {
        arg
    }
    .into()
}

pub static PR_FETCH_BRANCH_NAME_FLAG: Flag<'static> = Flag {
    short: "-b=",
    long: "--branch-name=",
    description: "Choose local name for the branch belonging to the preceding pull request",
};

pub static PR_FETCH_CHECKOUT_FLAG: Flag<'static> = Flag {
    short: "-c",
    long: "--checkout",
    description: "Check out the branch belonging to the first pull request",
};

pub static PR_FETCH_REPO_NAME_FLAG: Flag<'static> = Flag {
    short: "-r=",
    long: "--repo-name=",
    description:
        "Choose a github repository, using the `origin` remote of the current repository by default",
};

pub static PR_FETCH_FLAGS: &[&Flag<'static>; 5] = &[
    &PR_FETCH_BRANCH_NAME_FLAG,
    &PR_FETCH_CHECKOUT_FLAG,
    &PR_FETCH_REPO_NAME_FLAG,
    &HELP_FLAG,
    &VERSION_FLAG,
];

pub async fn pr_fetch(args: &CommandArgs) -> anyhow::Result<()> {
    let checkout_flag =
        args.contains(PR_FETCH_CHECKOUT_FLAG.short) || args.contains(PR_FETCH_CHECKOUT_FLAG.long);

    let mut args = args.iter().peekable();

    let mut pull_requests_with_maybe_custom_branch_names = vec![];

    let mut remote_name: Option<String> = None;

    let mut no_more_flags = false;

    while let Some(arg) = args.next() {
        // After "--", each argument is interpreted literally. This way, we can e.g. use filenames that are named exactly the same as flags
        if arg == "--" {
            no_more_flags = true;
            continue;
        };

        if let Some(flag) = PR_FETCH_REPO_NAME_FLAG.extract_from_arg(arg) {
            remote_name = Some(flag);
            continue;
        }

        if arg.starts_with('-') && !no_more_flags {
            if !is_valid_flag(arg, PR_FETCH_FLAGS) {
                fail!("Invalid flag: {arg}");
                let _ = help(Some("pr-fetch"));
                std::process::exit(1);
            }

            // Do not consider flags as arguments
            continue;
        }

        let arg = ignore_octothorpe(arg);

        let (pull_request, hash) = parse_if_maybe_hash(&arg, "@");

        if !pull_request.chars().all(|ch| ch.is_numeric()) {
            fail!(
                "The following argument couldn't be parsed as a pull request number: {arg}
  Examples of valid pull request numbers (with custom commit hashes supported): 1154, 500, '1001@0b36296f67a80309243ea5c8892c79798c6dcf93'"
            );
            continue;
        }

        let next_arg = args.peek();
        let maybe_custom_branch_name: Option<String> = next_arg.and_then(|next_arg| {
            PR_FETCH_BRANCH_NAME_FLAG
                .extract_from_arg(next_arg)
                .filter(|branch_name| is_valid_branch_name(branch_name))
        });

        if maybe_custom_branch_name.is_some() {
            args.next();
        };

        pull_requests_with_maybe_custom_branch_names.push((
            pull_request,
            maybe_custom_branch_name,
            hash,
        ));
    }

    // The user hasn't provided a custom remote, so we're going to try `origin`
    if remote_name.is_none() {
        let remote = GIT(&["remote", "get-url", "origin"])?;
        if remote.starts_with(GITHUB_REMOTE_PREFIX) && remote.ends_with(GITHUB_REMOTE_SUFFIX) {
            let start = GITHUB_REMOTE_PREFIX.len();
            let end = remote.len() - GITHUB_REMOTE_SUFFIX.len();
            remote_name = remote.get(start..end).map(|remote| remote.into());
        };
    }

    let Some(remote_name) = remote_name else {
        return Err(anyhow!(
            "Could not get the remote, it should be in the form e.g. helix-editor/helix.",
        ));
    };

    let client = reqwest::Client::new();

    for (i, (pull_request, maybe_custom_branch_name, hash)) in
        pull_requests_with_maybe_custom_branch_names
            .iter()
            .enumerate()
    {
        match fetch_pull_request(
            &remote_name,
            pull_request,
            &client,
            maybe_custom_branch_name.as_deref(),
            hash.as_deref(),
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
                            pull_request.bright_blue(),
                            " ".bright_blue(),
                            response.title.bright_blue().italic()
                        ),
                        &response.html_url
                    ),
                    info.branch.local_branch_name.bright_cyan(),
                    hash.clone()
                        .map(|commit_hash| format!(", at commit {}", commit_hash.bright_yellow()))
                        .unwrap_or_default()
                );

                // Attempt to cleanup after ourselves
                let _ = GIT(&["remote", "remove", &info.remote.local_remote_alias]);

                // If user uses --checkout flag, we're going to checkout the first PR only
                if i == 0 && checkout_flag {
                    if let Err(cant_checkout) = GIT(&["checkout", &info.branch.local_branch_name]) {
                        fail!(
                            "Could not check out branch {}:\n{cant_checkout}",
                            info.branch.local_branch_name
                        )
                    } else {
                        success!(
                            "Automatically checked out the first branch: {}",
                            info.branch.local_branch_name
                        )
                    }
                }
            }
            Err(err) => {
                fail!("{err}");
                continue;
            }
        };
    }

    Ok(())
}
