use std::fs;
use std::path::PathBuf;

use anyhow::bail;

use crate::git_commands::{Commit, GIT, GIT_ROOT};
use crate::utils::normalize_commit_msg;
use crate::{CONFIG_ROOT, success};

pub fn gen_patch(commit: Commit, filename: Option<PathBuf>) -> anyhow::Result<()> {
    let config_path = GIT_ROOT.join(CONFIG_ROOT.as_str());

    if !config_path.exists() {
        success!(
            "Config directory {} does not exist, creating it...",
            config_path.to_string_lossy()
        );
        fs::create_dir_all(&config_path)?;
    }

    // 1. if the user provides a custom filename for the patch file, use that
    // 2. otherwise use the commit message
    // 3. if all fails use the commit hash
    let patch_filename = filename.map_or_else(
        || {
            GIT(&["log", "--format=%B", "--max-count=1", commit.as_ref()]).map_or_else(
                |_| commit.clone().into_inner(),
                |commit_msg| normalize_commit_msg(&commit_msg),
            )
        },
        |filename| filename.to_str().unwrap_or_default().to_string(),
    );

    let patch_filename = format!("{patch_filename}.patch");

    let patch_file_path = config_path.join(&patch_filename);

    // Paths are UTF-8 encoded. If we cannot convert to UTF-8 that means it is not a
    // valid path
    let Some(patch_file_path_str) = patch_file_path.as_os_str().to_str() else {
        bail!("Not a valid path: {patch_file_path:?}");
    };

    if let Err(err) = GIT(&[
        "format-patch",
        "-1",
        &commit.clone().into_inner(),
        "--output",
        patch_file_path_str,
    ]) {
        bail!(
            "Could not get patch output for patch {}\n{err}",
            commit.into_inner()
        );
    }

    success!(
        "Created patch file at {}",
        patch_file_path.to_string_lossy()
    );

    Ok(())
}
