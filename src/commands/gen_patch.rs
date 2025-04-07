use std::fs;

use crate::cli::gen_patch::{GenPatch, Patch};
use crate::git_commands::{GIT, GIT_ROOT};
use crate::utils::normalize_commit_msg;
use crate::{CONFIG_ROOT, fail, success};

pub fn gen_patch(args: GenPatch) -> anyhow::Result<()> {
    let config_path = GIT_ROOT.join(CONFIG_ROOT.as_str());

    if !config_path.exists() {
        success!(
            "Config directory {} does not exist, creating it...",
            config_path.to_string_lossy()
        );
        fs::create_dir_all(&config_path)?;
    }

    for Patch {
        commit,
        custom_filename,
    } in args.patches
    {
        // 1. if the user provides a custom filename for the patch file, use that
        // 2. otherwise use the commit message
        // 3. if all fails use the commit hash
        let patch_filename = custom_filename.unwrap_or_else(|| {
            GIT(&["log", "--format=%B", "--max-count=1", &commit]).map_or_else(
                |_| commit.clone(),
                |commit_msg| normalize_commit_msg(&commit_msg),
            )
        });

        let patch_filename = format!("{patch_filename}.patch");

        let patch_file_path = config_path.join(&patch_filename);

        // Paths are UTF-8 encoded. If we cannot convert to UTF-8 that means it is not a
        // valid path
        let Some(patch_file_path_str) = patch_file_path.as_os_str().to_str() else {
            fail!("Not a valid path: {patch_file_path:?}");
            continue;
        };

        if let Err(err) = GIT(&[
            "format-patch",
            "-1",
            &commit,
            "--output",
            patch_file_path_str,
        ]) {
            fail!("Could not get patch output for patch {}\n{err}", commit);
            continue;
        }

        success!(
            "Created patch file at {}",
            patch_file_path.to_string_lossy()
        );
    }

    Ok(())
}
