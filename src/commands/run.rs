//! `run` subcommand

use crate::config::{Config, Ref};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Write as _;
use std::path::PathBuf;

use anyhow::{anyhow, bail};
use colored::Colorize as _;

use crate::git::{self, GIT_ROOT, git};
use crate::github_api::{Branch, Remote, RemoteBranch};
use crate::utils::{display_link, with_uuid};
use crate::{CONFIG_PATH, CONFIG_ROOT, confirm_prompt};

/// Backup for a file
struct FileBackup {
    /// Name of the file to backup in `.patchy` config directory
    filename: OsString,
    /// Contents of the backed up file
    contents: String,
}

/// Run patchy, if `yes` then there will be no prompt
pub async fn run(yes: bool) -> anyhow::Result<()> {
    let Some(config) = Config::read(yes)? else {
        // if it's Ok(None), we have wrote the default config
        return Ok(());
    };

    let Ref {
        item: remote_branch,
        commit,
    } = config.remote_branch;

    if config.repo.is_empty() {
        bail!(
            "You haven't specified a `repo` in your config, which can be for example:
  - `helix-editor/helix`
  - `microsoft/vscode`

  For more information see this guide: https://github.com/nik-rev/patchy/blob/main/README.md"
        );
    }

    // --- Backup all files in the `.patchy` config directory

    let config_files = fs::read_dir(&*CONFIG_PATH).map_err(|err| {
        anyhow!(
            "Failed to read files in directory `{}`:\n{err}",
            &CONFIG_PATH.display()
        )
    })?;

    let mut backed_up_files = Vec::new();

    for config_file in config_files.flatten() {
        let file_backup = fs::read_to_string(config_file.path())
            .map_err(|err| anyhow!("{err}"))
            .map(|contents| FileBackup {
                filename: config_file.file_name(),
                contents,
            })
            .map_err(|err| {
                anyhow!(
                    "failed to backup patchy config file {} for configuration files:\n{err}",
                    config_file.file_name().display()
                )
            })?;

        backed_up_files.push(file_backup);
    }

    // ---

    let info = RemoteBranch {
        remote: Remote {
            repository_url: format!("https://github.com/{}.git", config.repo),
            local_remote_alias: with_uuid(&config.repo),
        },
        branch: Branch {
            upstream_branch_name: remote_branch.clone(),
            local_branch_name: with_uuid(&remote_branch),
        },
    };

    git::add_remote_branch(&info, commit.as_ref())?;

    let previous_branch = git::checkout_from_remote(
        &info.branch.local_branch_name,
        &info.remote.local_remote_alias,
    )?;

    if config.pull_requests.is_empty() && config.branches.is_empty() {
        log::warn!(
            "You haven't specified any pull requests or branches to fetch in your config, {}",
            display_link(
                "see the instructions on how to configure patchy.",
                "https://github.com/nik-rev/patchy?tab=readme-ov-file#config"
            )
        );
    }

    // Process pull requests
    // TODO: make this concurrent, see https://users.rust-lang.org/t/processing-subprocesses-concurrently/79638/3
    // Git cannot handle multiple threads executing commands in the same repository,
    // so we can't use threads, but we can run processes in the background
    for Ref {
        item: pull_request,
        commit,
    } in &config.pull_requests
    {
        // TODO: refactor this to not use such deep nesting
        let Ok((response, info)) =
            git::fetch_pull_request(&config.repo, pull_request, None, commit.as_ref())
                .await
                .inspect_err(|err| {
                    log::error!("failed to fetch branch from remote:\n{err}");
                })
        else {
            continue;
        };

        if let Err(err) =
            git::merge_pull_request(&info, pull_request, &response.title, &response.html_url)
        {
            log::error!("failed to merge {pull_request}: {err}");
            continue;
        }

        log::info!(
            "Merged pull request {}",
            display_link(
                &format!(
                    "{}{}{}{}",
                    "#".bright_blue(),
                    pull_request.bright_blue(),
                    " ".bright_blue(),
                    &response.title.bright_blue().italic()
                ),
                &response.html_url
            ),
        );
    }

    // Process branches
    for Ref {
        item: branch_path,
        commit: commit_hash,
    } in &config.branches
    {
        // Parse the branch path into owner/repo/branch format
        let parts: Vec<&str> = branch_path.split('/').collect();
        if parts.len() < 3 {
            log::error!("Invalid branch format: {branch_path}. Expected format: owner/repo/branch");
            continue;
        }

        let owner = parts[0];
        let repo = parts[1];
        let branch_name = parts[2..].join("/");

        let remote = crate::cli::Remote {
            owner: owner.to_string(),
            repo: repo.to_string(),
            branch: branch_name.clone(),
        };

        let info = match git::fetch_branch(&remote, commit_hash.as_ref()).await {
            Ok((_, info)) => info,
            Err(err) => {
                log::error!("Could not fetch branch {owner}/{repo}/{branch_name}: {err}");
                continue;
            }
        };

        if let Err(err) = git::merge_into_main(
            &info.branch.local_branch_name,
            &info.branch.upstream_branch_name,
        ) {
            log::error!("{err}");
        }

        log::info!(
            "Merged branch {}/{}/{} {}",
            owner.bright_blue(),
            repo.bright_blue(),
            branch_name.bright_blue(),
            commit_hash
                .as_ref()
                .map(|hash| format!("at commit {}", hash.as_ref().bright_yellow()))
                .unwrap_or_default()
        );

        // Clean up the remote branch
        if let Err(err) = git::delete_remote_and_branch(
            &info.remote.local_remote_alias,
            &info.branch.local_branch_name,
        ) {
            log::warn!("Failed to clean up branch: {err}");
        }
    }

    if let Err(err) = fs::create_dir_all(GIT_ROOT.join(CONFIG_ROOT.as_str())) {
        git(["checkout", &previous_branch])?;

        git::delete_remote_and_branch(
            &info.remote.local_remote_alias,
            &info.branch.local_branch_name,
        )?;

        bail!("Could not create directory {}\n{err}", CONFIG_ROOT.as_str());
    }

    // Restore all the backup files

    for FileBackup {
        filename, contents, ..
    } in &backed_up_files
    {
        let path = GIT_ROOT.join(PathBuf::from(CONFIG_ROOT.as_str()).join(filename));
        let mut file =
            File::create(&path).map_err(|err| anyhow!("failed to restore backup: {err}"))?;

        write!(file, "{contents}")?;
    }

    // apply patches if they exist

    for patch in config.patches {
        let file_name = GIT_ROOT
            .join(CONFIG_ROOT.as_str())
            .join(format!("{patch}.patch"));

        if !file_name.exists() {
            log::warn!("Could not find patch {patch}, skipping");
            continue;
        }

        if let Err(err) = git(["am", "--keep-cr", "--signoff", &file_name.to_string_lossy()]) {
            git(["am", "--abort"])?;
            return Err(anyhow!("Could not apply patch {patch}, skipping\n{err}"));
        }

        let last_commit_message = git(["log", "-1", "--format=%B"])?;

        log::info!(
            "Applied patch {patch} {}",
            last_commit_message
                .lines()
                .next()
                .unwrap_or_default()
                .bright_blue()
                .italic()
        );
    }

    git(["add", CONFIG_ROOT.as_str()])?;
    git(["commit", "--message", "patchy: Restore configuration files"])?;

    let temporary_branch = with_uuid("temp-branch");

    git(["switch", "--create", &temporary_branch])?;

    git::delete_remote_and_branch(
        &info.remote.local_remote_alias,
        &info.branch.local_branch_name,
    )?;

    if yes
        || confirm_prompt!(
            "Overwrite branch {}? This is irreversible.",
            config.local_branch.cyan()
        )
    {
        // forcefully renames the branch we are currently on into the branch specified
        // by the user. WARNING: this is a destructive action which erases the
        // original branch
        git([
            "branch",
            "--move",
            "--force",
            &temporary_branch,
            &config.local_branch,
        ])?;
        if yes {
            log::info!(
                "Automatically overwrote branch {} since you supplied the {} flag",
                config.local_branch.cyan(),
                "--yes".bright_magenta()
            );
        }
        log::info!("Success!");
        return Ok(());
    }

    let overwrite_command = format!(
        "git branch --move --force {temporary_branch} {}",
        config.local_branch
    );
    log::info!(
        "You can still manually overwrite {} with:\n  {overwrite_command}\n",
        config.local_branch.cyan(),
    );

    Ok(())
}
