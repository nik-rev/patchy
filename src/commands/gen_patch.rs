use std::{fs, process};

use crate::CONFIG_ROOT;
use crate::{
    commands::help,
    fail,
    flags::{is_valid_flag, Flag},
    git_commands::{is_valid_branch_name, GIT, GIT_ROOT},
    success,
    types::CommandArgs,
    utils::normalize_commit_msg,
};

use super::help::{HELP_FLAG, VERSION_FLAG};

pub static GEN_PATCH_NAME_FLAG: Flag<'static> = Flag {
    short: "-n=",
    long: "--patch-filename=",
    description: "Choose filename for the patch",
};

pub static GEN_PATCH_FLAGS: &[&Flag<'static>; 3] =
    &[&GEN_PATCH_NAME_FLAG, &HELP_FLAG, &VERSION_FLAG];

pub fn gen_patch(args: &CommandArgs) -> anyhow::Result<()> {
    if args.is_empty() {
        fail!("You haven't specified any commit hashes");
        help(Some("gen-patch"))?;
    }
    let mut args = args.iter().peekable();
    let mut commit_hashes_with_maybe_custom_patch_filenames = vec![];

    let config_path = GIT_ROOT.join(CONFIG_ROOT);

    let mut no_more_flags = false;

    while let Some(arg) = args.next() {
        // After "--", each argument is interpreted literally. This way, we can e.g. use filenames that are named exactly the same as flags
        if arg == "--" {
            no_more_flags = true;
            continue;
        };

        if arg.starts_with('-') && !no_more_flags {
            if !is_valid_flag(arg, GEN_PATCH_FLAGS) {
                fail!("Invalid flag: {arg}");
                let _ = help(Some("gen-patch"));
                process::exit(1);
            }

            // Do not consider flags as arguments
            continue;
        }

        // Only merge commits can have 2 or more parents
        let is_merge_commit = GIT(&["rev-parse", &format!("{}^2", arg)]).is_ok();

        if is_merge_commit {
            fail!(
                "Commit {} is a merge commit, which cannot be turned into a .patch file",
                arg
            );

            continue;
        }

        let next_arg = args.peek();
        let maybe_custom_patch_filename: Option<String> = next_arg.and_then(|next_arg| {
            GEN_PATCH_NAME_FLAG
                .extract_from_arg(next_arg)
                .filter(|branch_name| is_valid_branch_name(branch_name))
        });

        if maybe_custom_patch_filename.is_some() {
            args.next();
        };

        commit_hashes_with_maybe_custom_patch_filenames.push((arg, maybe_custom_patch_filename));
    }

    if !config_path.exists() {
        success!(
            "Config directory {} does not exist, creating it...",
            config_path.to_string_lossy()
        );
        fs::create_dir_all(&config_path)?;
    }

    for (patch_commit_hash, maybe_custom_patch_name) in
        commit_hashes_with_maybe_custom_patch_filenames
    {
        // 1. if the user provides a custom filename for the patch file, use that
        // 2. otherwise use the commit message
        // 3. if all fails use the commit hash
        let patch_filename = maybe_custom_patch_name.unwrap_or_else(|| {
            GIT(&["log", "--format=%B", "--max-count=1", patch_commit_hash]).map_or_else(
                |_| patch_commit_hash.to_owned(),
                |commit_msg| normalize_commit_msg(&commit_msg),
            )
        });

        let patch_filename = format!("{patch_filename}.patch");

        let patch_file_path = config_path.join(&patch_filename);

        // Paths are UTF-8 encoded. If we cannot convert to UTF-8 that means it is not a valid path
        let Some(patch_file_path_str) = patch_file_path.as_os_str().to_str() else {
            fail!("Not a valid path: {patch_file_path:?}");
            continue;
        };

        if let Err(err) = GIT(&[
            "format-patch",
            "-1",
            patch_commit_hash,
            "--output",
            patch_file_path_str,
        ]) {
            fail!(
                "Could not get patch output for patch {}\n{err}",
                patch_commit_hash
            );
            continue;
        };

        success!(
            "Created patch file at {}",
            patch_file_path.to_string_lossy()
        );
    }

    Ok(())
}
