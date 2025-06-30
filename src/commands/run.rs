//! `run` subcommand

use crate::config::{self, BranchName, Config, PrNumber, PullRequest};
use anyhow::Result;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Write as _;
use std::path::PathBuf;

use anyhow::{anyhow, bail};
use colored::Colorize as _;

use crate::github::{self, Branch, Remote, RemoteBranch};
use crate::utils::{format_pr, format_url, with_uuid};
use crate::{commands, confirm_prompt, git};

/// Backup for a file
struct FileBackup {
    /// Name of the file to backup in `.patchy` config directory
    filename: OsString,
    /// Contents of the backed up file
    contents: String,
}

/// Run patchy, if `yes` then there will be no prompt
pub async fn run(yes: bool, use_gh_cli: bool) -> Result<()> {
    let root = config::ROOT.as_str();

    let Ok(config_string) = fs::read_to_string(&*config::FILE_PATH) else {
        log::error!(
            "Could not find configuration file at {root}/{}",
            config::FILE
        );

        // We don't want to have *any* sort of prompt when using the -y flag since that
        // would be problematic in scripts
        if !yes && confirm_prompt!("Would you like us to run `patchy init` to initialize it?",) {
            commands::init()?;
        } else if yes {
            log::info!("You can create it with `patchy init`",);
        } else {
            // user said "no" in the prompt, so we don't do any initializing
        }

        // We don't want to read the default configuration file as config_string. Since
        // it's empty there's no reason why the user would want to run it.

        return Ok(());
    };

    log::trace!("Using configuration file {}", config::FILE_PATH.display());

    let config = toml::from_str::<Config>(&config_string).map_err(|err| {
        anyhow!(
            "Could not parse `{root}/{}` configuration file:\n{err}",
            config::FILE
        )
    })?;

    let config::Branch {
        name: remote_branch,
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

    let config_files = fs::read_dir(&*config::PATH).map_err(|err| {
        anyhow!(
            "Failed to read files in directory `{}`:\n{err}",
            &config::PATH.display()
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
            local_branch_name: BranchName::try_new(with_uuid(remote_branch.as_ref()))
                .expect("adding UUID to branch name does not invalidate it"),
        },
    };

    github::add_remote_branch(&info, commit.as_ref())?;

    // we want to checkout the `branch` of `remote`
    let branch = &info.branch.local_branch_name;
    let remote = &info.remote.local_remote_alias;

    let previous_branch = git::get_head_commit().map_err(|err| {
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

    if config.pull_requests.is_empty() && config.branches.is_empty() {
        log::warn!(
            "You haven't specified any pull requests or branches to fetch in your config, {}",
            format_url(
                "see the instructions on how to configure patchy.",
                "https://github.com/nik-rev/patchy?tab=readme-ov-file#config"
            )
        );
    }

    // Process pull requests
    // TODO: make this concurrent, see https://users.rust-lang.org/t/processing-subprocesses-concurrently/79638/3
    // Git cannot handle multiple threads executing commands in the same repository,
    // so we can't use threads, but we can run processes in the background
    for PullRequest {
        number: pull_request,
        commit,
    } in &config.pull_requests
    {
        // TODO: refactor this to not use such deep nesting
        let Ok((response, info)) = github::fetch_pull_request(
            &config.repo,
            *pull_request,
            None,
            commit.as_ref(),
            use_gh_cli,
        )
        .await
        .inspect_err(|err| {
            log::error!("failed to fetch branch from remote:\n{err}");
        }) else {
            continue;
        };

        if let Err(err) =
            merge_pull_request(&info, *pull_request, &response.title, &response.html_url)
        {
            log::error!("failed to merge {pull_request}: {err}");
            continue;
        }

        log::info!(
            "Merged pull request {}",
            format_pr(*pull_request, &response.title, &response.html_url),
        );
    }

    // Process branches
    for remote in &config.branches {
        let owner = &remote.owner;
        let repo = &remote.repo;
        let branch = &remote.branch;
        let Ok((_, info)) = github::fetch_branch(remote, use_gh_cli)
            .await
            .inspect_err(|err| {
                log::error!("failed to fetch branch {owner}/{repo}/{branch}: {err}");
            })
        else {
            continue;
        };

        if let Err(err) = merge(
            &info.branch.local_branch_name,
            &info.branch.upstream_branch_name,
        ) {
            log::error!("{err}");
        }

        log::info!(
            "Merged branch {}/{}/{} {}",
            owner.as_ref().bright_blue(),
            repo.as_ref().bright_blue(),
            branch.as_ref().bright_blue(),
            remote
                .commit
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

    if let Err(err) = fs::create_dir_all(git::ROOT.join(config::ROOT.as_str())) {
        git::checkout(&previous_branch)?;

        git::delete_remote_and_branch(
            &info.remote.local_remote_alias,
            &info.branch.local_branch_name,
        )?;

        bail!(
            "Could not create directory {}\n{err}",
            config::ROOT.as_str()
        );
    }

    // Restore all the backup files

    for FileBackup {
        filename, contents, ..
    } in &backed_up_files
    {
        let path = git::ROOT.join(PathBuf::from(config::ROOT.as_str()).join(filename));
        let mut file =
            File::create(&path).map_err(|err| anyhow!("failed to restore backup: {err}"))?;

        write!(file, "{contents}")?;
    }

    // apply patches if they exist

    for patch in config.patches {
        let file_name = git::ROOT
            .join(config::ROOT.as_str())
            .join(format!("{patch}.patch"));

        if !file_name.exists() {
            log::error!("failed to find patch {patch}, skipping");
            continue;
        }

        if let Err(err) = git::apply_patch(&file_name) {
            log::error!("failed to apply patch {patch}, skipping\n{err}");
            continue;
        }

        let last_commit_message = git::last_commit_message()?;

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

    git::add(config::ROOT.as_str())?;
    git::commit("restore configuration files")?;

    let temporary_branch = with_uuid("temp-branch");

    git::create_branch(&temporary_branch)?;

    git::delete_remote_and_branch(
        &info.remote.local_remote_alias,
        &info.branch.local_branch_name,
    )?;

    if yes
        || confirm_prompt!(
            "Overwrite branch {}? This is irreversible.",
            config.local_branch.as_ref().cyan()
        )
    {
        git::rename_branch(&temporary_branch, config.local_branch.as_ref())?;
        if yes {
            log::info!(
                "Automatically overwrote branch {} since you supplied the {} flag",
                config.local_branch.as_ref().cyan(),
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
        config.local_branch.as_ref().cyan(),
    );

    Ok(())
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
        let pr = format_pr(pull_request, pr_title, pr_url);

        let support_url = format_url(
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
