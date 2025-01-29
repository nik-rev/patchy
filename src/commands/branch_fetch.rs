use std::process;

use crate::{
    commands::help,
    fail,
    flags::{is_valid_flag, Flag},
    git_commands::is_valid_branch_name,
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
    repo: String,
    /// # Examples
    ///
    /// master
    branch: String,
    /// If specified, use a custom branch name instead of a generated one
    ///
    /// # Examples
    ///
    /// my-custom-branch123
    local_branch_name: Option<String>,
    /// If specified, do a **hard reset** to this commit when fetching the branch
    ///
    /// # Examples
    ///
    /// 6049f2035
    commit_hash: Option<String>,
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
}

pub fn branch_fetch(args: &CommandArgs) {
    let has_checkout_flag = BRANCH_FETCH_CHECKOUT_FLAG.is_in(args);

    let mut args = args.iter().peekable();

    let mut branches_with_maybe_custom_names = vec![];

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

        let (remote, hash) = parse_if_maybe_hash(arg, "@");

        let Some((repo, branch)) = remote.rsplit_once('/') else {
            fail!("Invalid format: {}, skipping. Valid format is: username/repo/branch. Example: helix-editor/helix/master", remote);

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

        branches_with_maybe_custom_names.push(Item::new(
            repo.to_owned(),
            branch.to_owned(),
            maybe_custom_branch_name,
            hash,
        ));
    }

    let client = reqwest::Client::new();
}
