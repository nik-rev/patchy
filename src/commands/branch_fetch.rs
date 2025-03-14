use std::process;

use colored::Colorize as _;

use crate::{
    commands::help,
    fail,
    flags::{Flag, is_valid_flag},
    git_commands::{GIT, fetch_branch, is_valid_branch_name},
    success,
    types::CommandArgs,
};

use super::run::parse_if_maybe_hash;

pub static BRANCH_FETCH_CHECKOUT_FLAG: Flag<'static> = Flag {
    short: "-c",
    long: "--checkout",
    description: "Check out the first fetched branch",
};

pub static BRANCH_FETCH_BRANCH_NAME_FLAG: Flag<'static> = Flag {
    short: "-b=",
    long: "--branch-name=",
    description: "Choose local name for the branch belonging to the preceding pull request",
};

pub static BRANCH_FETCH_FLAGS: &[&Flag<'static>; 2] =
    &[&BRANCH_FETCH_BRANCH_NAME_FLAG, &BRANCH_FETCH_CHECKOUT_FLAG];

pub struct Item {
    /// # Examples
    ///
    /// helix-editor/helix
    pub repo: String,
    /// # Examples
    ///
    /// master
    pub branch: String,
    /// If specified, use a custom branch name instead of a generated one
    ///
    /// # Examples
    ///
    /// my-custom-branch123
    pub local_branch_name: Option<String>,
    /// If specified, do a **hard reset** to this commit when fetching the branch
    ///
    /// # Examples
    ///
    /// 6049f2035
    pub commit_hash: Option<String>,
}

impl Item {
    pub fn new(
        repo: String,
        branch: String,
        local_branch_name: Option<String>,
        commit_hash: Option<String>,
    ) -> Self {
        Self {
            repo,
            branch,
            local_branch_name,
            commit_hash,
        }
    }

    pub fn create(arg: &str) -> anyhow::Result<Self> {
        let (remote, hash) = parse_if_maybe_hash(arg, "@");

        let (repo, branch) = remote.rsplit_once('/').ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid format: {}, skipping. \
Valid format is: username/repo/branch. Example: helix-editor/helix/master",
                remote
            )
        })?;

        Ok(Self::new(repo.to_owned(), branch.to_owned(), None, hash))
    }

    #[must_use]
    pub fn with_branch_name(mut self, branch_name: Option<String>) -> Self {
        self.local_branch_name = branch_name;
        self
    }
}

pub async fn branch_fetch(args: &CommandArgs) -> anyhow::Result<()> {
    if args.is_empty() {
        let _ = help(Some("branch-fetch"));
        process::exit(1);
    }

    let has_checkout_flag = BRANCH_FETCH_CHECKOUT_FLAG.is_in(args);

    let mut args = args.iter().peekable();

    let mut items = vec![];

    let mut no_more_flags = false;

    while let Some(arg) = args.next() {
        // After "--", each argument is interpreted literally. This way, we can e.g. use filenames that are named exactly the same as flags
        if arg == "--" {
            no_more_flags = true;
            continue;
        };

        if arg.starts_with('-') && !no_more_flags {
            if !is_valid_flag(arg, BRANCH_FETCH_FLAGS) {
                fail!("Invalid flag: {arg}");
                let _ = help(Some("branch-fetch"));
                process::exit(1);
            }

            // Do not consider flags as arguments
            continue;
        }

        let Ok(item) = Item::create(arg).map_err(|err| fail!("{err}")) else {
            continue;
        };

        let next_arg = args.peek();
        let maybe_custom_branch_name: Option<String> = next_arg.and_then(|next_arg| {
            BRANCH_FETCH_BRANCH_NAME_FLAG
                .extract_from_arg(next_arg)
                .filter(|branch_name| is_valid_branch_name(branch_name))
        });

        if maybe_custom_branch_name.is_some() {
            args.next();
        };

        let item = item.with_branch_name(maybe_custom_branch_name);

        items.push(item);
    }

    let client = reqwest::Client::new();

    for (i, item) in items.into_iter().enumerate() {
        let hash = item.commit_hash.clone();
        let repo = item.repo.clone();
        match fetch_branch(item, &client).await {
            Ok((_, info)) => {
                success!(
                    "Fetched branch {}/{} available at branch {}{}",
                    repo,
                    info.branch.upstream_branch_name,
                    info.branch.local_branch_name.bright_cyan(),
                    hash.map(|commit_hash| format!(", at commit {}", commit_hash.bright_yellow()))
                        .unwrap_or_default()
                );

                // Attempt to cleanup after ourselves
                let _ = GIT(&["remote", "remove", &info.remote.local_remote_alias]);

                // If user uses --checkout flag, we're going to checkout the first fetched branch
                if i == 0 && has_checkout_flag {
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
            }
            Err(err) => {
                fail!("{err}");
                continue;
            }
        };
    }

    Ok(())
}
