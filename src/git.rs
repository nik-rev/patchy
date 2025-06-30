//! Low-level API for git

use anyhow::Result;
use std::{
    env, io,
    path::{Path, PathBuf},
    process::{self, Output},
    sync::LazyLock,
};

use crate::config::{BranchName, CommitId};

/// Add the file
pub fn add(file: &str) -> Result<String> {
    git(["add", file])
}

/// Retrieve message of the last commit
pub fn last_commit_message() -> Result<String> {
    git(["log", "--format=%B", "--max-count=1"])
}

/// Retrieve message of specific commit
pub fn get_message_of_commit(commit: &str) -> Result<String> {
    git(["log", "--format=%B", "--max-count=1", commit])
}

/// Merge the branch into the current one
pub fn merge(branch: &str) -> Result<String> {
    git(["merge", "--squash", branch])
}

/// Remote the given remote
pub fn remove_remote(remote: &str) -> Result<String> {
    git(["remote", "remove", remote])
}

/// Checkout the commit
pub fn checkout(object: &str) -> Result<String> {
    git(["checkout", object])
}

/// Create a commit with the given message
pub fn commit(message: &str) -> Result<String> {
    git(["commit", "--message", &format!("patchy: {message}")])
}

/// Fetch remote `url` to local `name`
pub fn add_remote(name: &str, url: &str) -> Result<String> {
    git(["remote", "add", name, url])
}

/// Fetches the `remote_branch` as the name of `local_branch` from `url`
pub fn fetch_remote_branch(
    local_branch: &BranchName,
    remote_branch: &BranchName,
    url: &str,
) -> Result<String> {
    git(["fetch", url, &format!("{remote_branch}:{local_branch}")])
}

/// Formats the commit as a `patch` and saves it to the specified path
pub fn save_commit_as_patch(commit: &CommitId, output_path: &str) -> Result<String> {
    git([
        "format-patch",
        "-1",
        commit.as_ref(),
        "--output",
        output_path,
    ])
}

/// Obtain the URL for a remote
pub fn get_remote_url(remote: &str) -> Result<String> {
    git(["remote", "get-url", remote])
}

/// Apply a `patch` as a commit
pub fn apply_patch(filename: &Path) -> Result<()> {
    if let Err(err) = git(["am", "--keep-cr", "--signoff", &filename.to_string_lossy()]) {
        git(["am", "--abort"])?;
        return Err(err);
    }

    Ok(())
}

/// `true` if there are unstaged changes
pub fn is_worktree_dirty() -> bool {
    git(["diff", "--cached", "--quiet"]).is_err()
}

/// Get the current commit that we are on
pub fn get_head_commit() -> Result<String> {
    git(["rev-parse", "--abbrev-ref", "HEAD"])
}

// TODO: make sure we are on the "patchy" branch when running
// this dangerous command
/// Removes all uncommitted changes
pub fn nuke_worktree() -> Result<String> {
    git(["reset", "--hard"])
}

/// `true` if the object exists (e.g. commit or branch)
pub fn does_object_exist(branch: &str) -> bool {
    git(["rev-parse", "--verify", branch]).is_err()
}

/// Removes a remote and its branch
///
/// WARNING: Only call this function if the script created
/// the branch or if the user gave explicit permission
pub fn delete_remote_and_branch(remote: &str, branch: &BranchName) -> Result<()> {
    git(["branch", "--delete", "--force", branch.as_ref()])?;
    git(["remote", "remove", remote])?;
    Ok(())
}

/// Create a `branch` and check it out
pub fn create_branch(branch: &str) -> Result<String> {
    git(["switch", "--create", branch])
}

/// forcefully renames the branch we are currently on into the branch specified
/// by the user. WARNING: this is a destructive action which erases the
/// branch name if it conflicts
pub fn rename_branch(old: &str, new: &str) -> Result<String> {
    git(["branch", "--move", "--force", old, new])
}

/// Resets the `branch` to the specified `commit`
pub fn reset_branch_to_commit(branch: &BranchName, commit: &CommitId) -> Result<String> {
    git(["branch", "--force", branch.as_ref(), commit.as_ref()])
}

/// Run `git` with the given arguments, and get its output
fn git<const N: usize>(args: [&str; N]) -> Result<String> {
    log::debug!("$ git {}", args.join(" "));
    get_git_output(&spawn_git(&args, &ROOT)?, &args)
}

/// Get output of the git process
pub fn get_git_output(output: &Output, args: &[&str]) -> Result<String> {
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

/// Spawn a git process and collect its output
pub fn spawn_git(args: &[&str], git_dir: &Path) -> Result<Output, io::Error> {
    process::Command::new("git")
        .args(args)
        .current_dir(git_dir)
        .output()
}

/// Location of the root directory of Git
pub static ROOT: LazyLock<PathBuf> = LazyLock::new(|| {
    match (|| {
        let current_dir = env::current_dir()?;
        // traverses until it finds a directory with a .git folder
        // and reports the path to the directory
        let args = ["rev-parse", "--show-toplevel"];
        let root = spawn_git(&args, &current_dir)?;
        get_git_output(&root, &args).map(Into::into)
    })() {
        Ok(root) => root,
        Err(err) => {
            log::error!("Failed to determine Git root directory.\n{err}");
            process::exit(1)
        }
    }
});
