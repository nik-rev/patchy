use crate::{fail, types::Repo, utils::display_link};
use colored::Colorize as _;
use std::{
    env, io,
    path::{Path, PathBuf},
    process::{self, Output},
};

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use reqwest::Client;

use crate::{
    trace,
    types::{Branch, BranchAndRemote, GitHubResponse, Remote},
    utils::{make_request, normalize_commit_msg, with_uuid},
};

/// A valid branch name consists of alphanumeric characters, but also '.', '-', '/' or '_'
pub fn is_valid_branch_name(branch_name: &str) -> bool {
    branch_name
        .chars()
        .all(|ch| ch.is_alphanumeric() || ch == '.' || ch == '-' || ch == '/' || ch == '_')
}

pub static GITHUB_REMOTE_PREFIX: &str = "git@github.com:";
pub static GITHUB_REMOTE_SUFFIX: &str = ".git";

pub fn spawn_git(args: &[&str], git_dir: &Path) -> Result<Output, io::Error> {
    process::Command::new("git")
        .args(args)
        .current_dir(git_dir)
        .output()
}

pub fn get_git_output(output: &Output, args: &[&str]) -> anyhow::Result<String> {
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_owned())
    } else {
        Err(anyhow::anyhow!(
            "Git command failed.\nCommand: git {}\nStdout: {}\nStderr: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ))
    }
}

pub fn get_git_root() -> anyhow::Result<PathBuf> {
    let current_dir = env::current_dir()?;

    let args = ["rev-parse", "--show-toplevel"];

    let root = spawn_git(&args, &current_dir)?;

    get_git_output(&root, &args).map(Into::into)
}

pub static GIT_ROOT: Lazy<PathBuf> = Lazy::new(|| match get_git_root() {
    Ok(root) => root,
    Err(err) => {
        fail!("Failed to determine Git root directory.\n{err}");
        process::exit(1)
    }
});

type Git = Lazy<Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>>;

pub static GIT: Git = Lazy::new(|| {
    Box::new(move |args: &[&str]| -> Result<String> {
        trace!("$ git {}", args.join(" "));
        get_git_output(&spawn_git(args, &GIT_ROOT)?, args)
    })
});

/// Fetches a branch of a remote into local. Optionally accepts a commit hash for versioning.
pub fn add_remote_branch(info: &BranchAndRemote, commit_hash: Option<&str>) -> anyhow::Result<()> {
    if let Err(err) = GIT(&[
        "remote",
        "add",
        &info.remote.local_remote_alias,
        &info.remote.repository_url,
    ]) {
        GIT(&["remote", "remove", &info.remote.local_remote_alias])?;
        return Err(anyhow!("Could not fetch remote: {err}"));
    }

    trace!(
        "Added remote {} for repository {}",
        &info.remote.repository_url,
        &info.remote.local_remote_alias
    );

    if let Err(err) = GIT(&[
        "fetch",
        &info.remote.repository_url,
        &format!(
            "{}:{}",
            info.branch.upstream_branch_name, info.branch.local_branch_name
        ),
    ]) {
        return Err(anyhow!(
            "We couldn't find branch {} of GitHub repository {}. Are you sure it \
                     exists?\n{err}",
            info.branch.upstream_branch_name,
            info.remote.repository_url
        ));
    }

    trace!(
        "Fetched branch {} as {} from repository {}",
        info.branch.upstream_branch_name,
        info.branch.local_branch_name,
        &info.remote.repository_url
    );

    if let Some(commit_hash) = commit_hash {
        GIT(&[
            "branch",
            "--force",
            &info.branch.local_branch_name,
            commit_hash,
        ])
        .map_err(|err| {
            anyhow!(
                "We couldn't find commit {} \
                                of branch {}. Are you sure it exists?\n{err}",
                commit_hash,
                info.branch.local_branch_name
            )
        })?;

        trace!("...and did a hard reset to commit {commit_hash}",);
    };

    Ok(())
}

/// Removes a remote and its branch
pub fn clean_up_remote(remote: &str, branch: &str) -> anyhow::Result<()> {
    // NOTE: Caller needs to ensure this function only runs if the script created the branch or if the user gave explicit permission
    GIT(&["branch", "--delete", "--force", branch])?;
    GIT(&["remote", "remove", remote])?;
    Ok(())
}

pub fn checkout_from_remote(branch: &str, remote: &str) -> anyhow::Result<String> {
    let current_branch = GIT(&["rev-parse", "--abbrev-ref", "HEAD"]).or_else(|err| {
        clean_up_remote(remote, branch)?;
        Err(anyhow!(
            "Couldn't get the current branch. This usually happens \
            when the current branch does not have any commits.\n{err}"
        ))
    })?;

    if let Err(err) = GIT(&["checkout", branch]) {
        clean_up_remote(remote, branch)?;
        return Err(anyhow!(
            "Could not checkout branch: {branch}, which belongs to remote {remote}\n{err}"
        ));
    };

    Ok(current_branch)
}

pub fn merge_into_main(
    local_branch: &str,
    remote_branch: &str,
) -> anyhow::Result<String, anyhow::Error> {
    trace!("Merging branch {local_branch}");

    if let Err(err) = GIT(&["merge", "--squash", local_branch]) {
        // nukes the worktree
        GIT(&["reset", "--hard"])?;
        return Err(anyhow!("Could not merge {remote_branch}\n{err}"));
    };

    // --squash will NOT commit anything. So we need to make it manually
    GIT(&[
        "commit",
        "--message",
        &format!("patchy: Merge {local_branch}",),
    ])?;

    Ok(format!("Merged {remote_branch} successfully"))
}

pub async fn merge_pull_request(
    info: BranchAndRemote,
    pull_request: &str,
    pr_title: &str,
    pr_url: &str,
) -> anyhow::Result<()> {
    merge_into_main(
        &info.branch.local_branch_name,
        &info.branch.upstream_branch_name,
    )
    .map_err(|err| {
        let pr = display_link(
            &format!(
                "{}{}{}{}",
                "#".bright_blue(),
                pull_request.bright_blue(),
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
            "Could not merge branch {} into the current branch for pull request {pr} \
            since the merge is non-trivial.\nYou will need to merge it yourself:\n  {} \
            {0}\nNote: To learn how to merge only once and re-use for subsequent \
            invocations of patchy, see {support_url}\nSkipping this PR. Error \
             message from git:\n{err}",
            &info.branch.local_branch_name.bright_cyan(),
            "git merge --squash".bright_blue()
        )
    })?;

    let has_unstaged_changes = GIT(&["diff", "--cached", "--quiet"]).is_err();

    if has_unstaged_changes {
        GIT(&[
            "commit",
            "--message",
            &format!(
                "patchy: auto-merge pull request {}",
                &pr_url.replace("github.com", "redirect.github.com")
            ),
        ])?;
    }

    clean_up_remote(
        &info.remote.local_remote_alias,
        &info.branch.local_branch_name,
    )?;

    Ok(())
}

enum AvailableBranch {
    /// In this case, we can just use the original `branch` that we passed in
    First,
    /// The first branch was available, so we slapped on some arbitrary identifier at the end
    /// Represents a branch like some-branch-2, some-branch-3
    Other(String),
}

/// Given a branch, either return this branch or the first available branch with an identifier at the end (a `-#`) where `#` represents a number
/// So we can keep on "trying" for a branch that isn't used. We might try `some-branch`, and if it already exists we will then try:
///
/// - some-branch-2
/// - some-branch-3
/// - some-branch-4
/// - ...
///
/// Stopping when we find the first available
///
/// We do not want to return a branch if it already exists, since we don't want to overwrite any branch potentially losing the user their work
///
/// We also don't want to ask for a prompt for a custom name, as it would be pretty annoying to specify a name for each branch if you have like 30 pull requests you want to merge
fn first_available_branch(branch: &str) -> AvailableBranch {
    let branch_exists = GIT(&["rev-parse", "--verify", branch]).is_err();

    if branch_exists {
        return AvailableBranch::First;
    }

    // the first number for which the branch does not exist
    let number = (2..)
        .find(|current| {
            let branch_with_num = format!("{}-{branch}", current);
            GIT(&["rev-parse", "--verify", &branch_with_num]).is_err()
        })
        .expect("There will eventually be a #-branch which is available.");

    let branch_name = format!("{number}-{branch}");

    AvailableBranch::Other(branch_name)
}

pub async fn fetch_branch(
    repo: &str,
    client: &Client,
    branch_name: &str,
    custom_branch_name: Option<&str>,
    commit_hash: Option<&str>,
) -> anyhow::Result<(Repo, BranchAndRemote)> {
    let url = format!("https://api.github.com/repos/{repo}");

    let response = make_request(client, &url)
        .await
        .map_err(|err| anyhow!("Could not fetch branch: {repo}\n{err}\n"))?;

    let response: Repo = serde_json::from_str(&response).map_err(|err| {
        anyhow!("Could not parse response.\n{response}. Could not parse because: \n{err}")
    })?;

    let info = BranchAndRemote {
        branch: Branch {
            upstream_branch_name: branch_name.to_owned(),
            local_branch_name: custom_branch_name.map_or_else(
                || {
                    let branch_name = &format!("{repo}/{branch_name}");

                    match first_available_branch(branch_name) {
                        AvailableBranch::First => branch_name.to_string(),
                        AvailableBranch::Other(branch) => branch,
                    }
                },
                Into::into,
            ),
        },
        remote: Remote {
            repository_url: response.clone_url.clone(),
            local_remote_alias: with_uuid(repo),
        },
    };

    add_remote_branch(&info, commit_hash)
        .map_err(|err| anyhow!("Could not add remote branch {repo}, skipping.\n{err}"))?;

    Ok((response, info))
}

pub async fn fetch_pull_request(
    repo: &str,
    pull_request: &str,
    client: &Client,
    custom_branch_name: Option<&str>,
    commit_hash: Option<&str>,
) -> anyhow::Result<(GitHubResponse, BranchAndRemote)> {
    let url = format!("https://api.github.com/repos/{}/pulls/{pull_request}", repo);

    let response = make_request(client, &url)
        .await
        .map_err(|err| anyhow!("Could not fetch pull request #{pull_request}\n{err}\n"))?;

    let response: GitHubResponse = serde_json::from_str(&response).map_err(|err| {
        anyhow!("Could not parse response.\n{response}. Could not parse because: \n{err}")
    })?;

    let info = BranchAndRemote {
        branch: Branch {
            upstream_branch_name: response.head.r#ref.clone(),
            local_branch_name: custom_branch_name.map_or_else(
                || {
                    let branch_name = &format!("{pull_request}/{}", &response.head.r#ref);

                    match first_available_branch(branch_name) {
                        AvailableBranch::First => branch_name.to_string(),
                        AvailableBranch::Other(branch) => branch,
                    }
                },
                Into::into,
            ),
        },
        remote: Remote {
            repository_url: response.head.repo.clone_url.clone(),
            local_remote_alias: with_uuid(&format!(
                "{title}-{}",
                pull_request,
                title = normalize_commit_msg(&response.html_url)
            )),
        },
    };

    add_remote_branch(&info, commit_hash).map_err(|err| {
        anyhow!("Could not add remote branch for pull request #{pull_request}, skipping.\n{err}")
    })?;

    Ok((response, info))
}
