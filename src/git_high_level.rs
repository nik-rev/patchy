//! Utilities for interacting with Git via spawning processes
//!
//! TODO:
//! - Extract into a separate module, put it behind some more nice API
//! - Use `gix`? Or anyways, we could go without spawning an entire process each
//!   time we want to interact with Git
use crate::config::{BranchName, CommitId, PrNumber};
use crate::git;

use anyhow::{Result, anyhow, bail};
use colored::Colorize as _;

use crate::github_api::RemoteBranch;
use crate::utils::display_link;

/// Fetches a branch of a remote into local. Optionally accepts a commit hash
/// for versioning.
pub fn add_remote_branch(remote_branch: &RemoteBranch, commit: Option<&CommitId>) -> Result<()> {
    git::add_remote(
        &remote_branch.remote.local_remote_alias,
        &remote_branch.remote.repository_url,
    )
    .map_err(|err| anyhow!("failed to fetch remote: {err}"))?;

    if let Err(err) = git::fetch_remote_branch(
        &remote_branch.branch.local_branch_name,
        &remote_branch.branch.upstream_branch_name,
        &remote_branch.remote.repository_url,
    ) {
        bail!(
            "Failed to find branch {} of GitHub repository {}. Are you sure it exists?\n{err}",
            remote_branch.branch.upstream_branch_name,
            remote_branch.remote.repository_url
        );
    }

    if let Some(commit) = commit {
        git::reset_branch_to_commit(&remote_branch.branch.local_branch_name, commit).map_err(
            |err| {
                anyhow!(
                    "Failed to find commit {} of branch {}. Are you sure the commit exists?\n{err}",
                    commit.as_ref(),
                    remote_branch.branch.local_branch_name
                )
            },
        )?;
    }

    Ok(())
}

/// Checkout `branch` of `remote`
pub fn checkout_from_remote(branch: &BranchName, remote: &str) -> Result<String> {
    let current_branch = git::get_head_commit().map_err(|err| {
        if let Err(err) = git::delete_remote_and_branch(remote, branch) {
            err
        } else {
            anyhow!(
                "Couldn't get the current branch. This usually happens \
            when the current branch does \
             not have any commits.\n{err}"
            )
        }
    })?;

    if let Err(err) = git::checkout(branch.as_ref()) {
        git::delete_remote_and_branch(remote, branch)?;
        bail!("Failed to checkout branch: {branch}, which belongs to remote {remote}\n{err}");
    }

    Ok(current_branch)
}

/// Create a merge commit that merges the `other_branch` into `current_branch`
pub fn merge(
    current_branch: &BranchName,
    other_branch: &BranchName,
) -> Result<String, anyhow::Error> {
    log::trace!("Merging branch {current_branch}");

    if let Err(err) = git::merge(current_branch.as_ref()) {
        git::nuke_worktree()?;
        bail!("failed to merge {other_branch}\n{err}");
    }

    // --squash will NOT commit anything. So we need to make the commit it manually
    git::commit(&format!("Merge {current_branch}"))?;

    Ok(format!("Merged {other_branch} successfully"))
}

/// Merge the `pull_request` into patchy's branch
pub fn merge_pull_request(
    info: &RemoteBranch,
    pull_request: PrNumber,
    pr_title: &str,
    pr_url: &str,
) -> Result<()> {
    merge(
        &info.branch.local_branch_name,
        &info.branch.upstream_branch_name,
    )
    .map_err(|err| {
        let pr = display_link(
            &format!(
                "{}{}{}{}",
                "#".bright_blue(),
                pull_request.to_string().bright_blue(),
                " ".bright_blue(),
                pr_title.bright_blue().italic()
            ),
            pr_url,
        );

        let support_url = display_link(
            "Merge conflicts (github)",
            "https://github.com/nik-rev/patchy?tab=readme-ov-file#merge-conflicts",
        )
        .bright_blue();

        anyhow!(
            "Could not merge branch {} into the current branch for pull request {pr} since the \
             merge is non-trivial.\nYou will need to merge it yourself:\n  {} {0}\nNote: To learn \
             how to merge only once and re-use for subsequent invocations of patchy, see \
             {support_url}\nSkipping this PR. Error message from git:\n{err}",
            &info.branch.local_branch_name.as_ref().bright_cyan(),
            "git merge --squash".bright_blue()
        )
    })?;

    if git::is_worktree_dirty() {
        git::commit(&format!(
            "auto-merge pull request {}",
            &pr_url.replace("github.com", "redirect.github.com")
        ))?;
    }

    git::delete_remote_and_branch(
        &info.remote.local_remote_alias,
        &info.branch.local_branch_name,
    )?;

    Ok(())
}

/// Available branch name to use
pub enum AvailableBranch {
    /// In this case, we can just use the original `branch` that we passed in
    First,
    /// The first branch was available, so we slapped on some arbitrary
    /// identifier at the end Represents a branch like some-branch-2,
    /// some-branch-3
    Other(BranchName),
}

/// Given a branch, either return this branch or the first available branch with
/// an identifier at the end (a `-#`) where `#` represents a number
/// So we can keep on "trying" for a branch that isn't used. We might try
/// `some-branch`, and if it already exists we will then try:
///
/// - some-branch-2
/// - some-branch-3
/// - some-branch-4
/// - ...
///
/// Stopping when we find the first available
///
/// We do not want to return a branch if it already exists, since we don't want
/// to overwrite any branch potentially losing the user their work
///
/// We also don't want to ask for a prompt for a custom name, as it would be
/// pretty annoying to specify a name for each branch if you have like 30 pull
/// requests you want to merge
pub fn find_first_available_branch(branch: &str) -> AvailableBranch {
    if git::does_object_exist(branch) {
        return AvailableBranch::First;
    }

    // the first number for which the branch does not exist
    #[expect(
        clippy::maybe_infinite_iter,
        reason = "there is definitely not an infinite number of branches"
    )]
    let number = (2..)
        .find(|current| git::does_object_exist(&format!("{current}-{branch}")))
        .expect("There will eventually be a #-branch which is available.");

    let branch_name = BranchName::try_new(format!("{number}-{branch}"))
        .expect("existing git branch is a valid branch name");

    AvailableBranch::Other(branch_name)
}
