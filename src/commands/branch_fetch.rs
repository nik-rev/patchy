use colored::Colorize as _;

use crate::cli::branch_fetch::BranchFetch;
use crate::git_commands::{GIT, fetch_branch};
use crate::{fail, success};

pub async fn branch_fetch(args: BranchFetch) -> anyhow::Result<()> {
    for (i, branch) in args.branches.into_iter().enumerate() {
        match fetch_branch(&branch).await {
            Ok((_, info)) => {
                success!(
                    "Fetched branch {}/{}/{} available at branch {}{}",
                    branch.repo_owner,
                    branch.repo_name,
                    info.branch.upstream_branch_name,
                    info.branch.local_branch_name.bright_cyan(),
                    branch
                        .commit
                        .map(|commit_hash| format!(", at commit {}", commit_hash.bright_yellow()))
                        .unwrap_or_default()
                );

                // Attempt to cleanup after ourselves
                let _ = GIT(&["remote", "remove", &info.remote.local_remote_alias]);

                // If user uses --checkout flag, we're going to checkout the
                // first fetched branch
                if i == 0 && args.checkout {
                    if let Err(cant_checkout) = GIT(&["checkout", &info.branch.local_branch_name]) {
                        fail!(
                            "Could not check out branch
                {}:\n{cant_checkout}",
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
