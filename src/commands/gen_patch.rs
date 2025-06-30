//! `gen-patch` subcommand

use std::fs;
use std::path::PathBuf;

use anyhow::bail;

use crate::config::{self, CommitId, PatchName};
use crate::git;
use crate::utils::normalize_commit_msg;

/// Generate patch `filename` at the given `Commit`
pub fn gen_patch(commit: CommitId, filename: Option<PatchName>) -> anyhow::Result<()> {
    if !config::PATH.exists() {
        log::info!(
            "Config directory {} does not exist, creating it...",
            config::PATH.to_string_lossy()
        );
        fs::create_dir_all(&*config::PATH)?;
    }

    // 1. if the user provides a custom filename for the patch file, use that
    // 2. otherwise use the commit message
    // 3. if all fails use the commit hash
    let patch_filename = filename.unwrap_or_else(|| {
        git::get_message_of_commit(commit.as_ref()).map_or_else(
            |_| {
                PatchName::try_new(commit.clone().into_inner().into()).expect("commit is not empty")
            },
            |commit_msg| {
                PatchName::try_new(PathBuf::from(normalize_commit_msg(&commit_msg)))
                    .expect("normalized commit message is not empty")
            },
        )
    });

    let patch_file_path = config::PATH.join(format!("{patch_filename}.patch"));

    // Paths are UTF-8 encoded. If we cannot convert to UTF-8 that means it is not a
    // valid path
    let Some(patch_file_path_str) = patch_file_path.as_os_str().to_str() else {
        bail!("invalid path: {patch_file_path:?}");
    };

    if let Err(err) = git::save_commit_as_patch(&commit, patch_file_path_str) {
        bail!(
            "failed to get patch output for patch {}\n{err}",
            commit.into_inner()
        );
    }

    log::info!(
        "Created patch file at {}",
        patch_file_path.to_string_lossy()
    );

    Ok(())
}
