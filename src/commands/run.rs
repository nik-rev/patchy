//! `run` subcommand

use std::fs::{self, File};
use std::io::Write as _;
use std::path::PathBuf;

use anyhow::{anyhow, bail};
use colored::Colorize as _;

use crate::commands::pr_fetch::ignore_octothorpe;
use crate::git::{self, GIT_ROOT, git};
use crate::github_api::{Branch, BranchAndRemote, Configuration, Remote};
use crate::utils::{display_link, parse_if_maybe_hash, with_uuid};
use crate::{CONFIG_FILE, CONFIG_ROOT, commands, confirm_prompt};

/// Run patchy, if `yes` then there will be no prompt
pub async fn run(yes: bool) -> anyhow::Result<()> {
    let root = CONFIG_ROOT.as_str();
    let config_path = GIT_ROOT.join(root);

    let config_file_path = config_path.join(CONFIG_FILE);

    let Ok(config_raw) = fs::read_to_string(config_file_path.clone()) else {
        log::error!("Could not find configuration file at {root}/{CONFIG_FILE}",);

        // We don't want to have *any* sort of prompt when using the -y flag since that
        // would be problematic in scripts
        if !yes && confirm_prompt!("Would you like us to run `patchy init` to initialize it?",) {
            commands::init()?;
        } else if yes {
            log::info!("You can create it with `patchy init`",);
        } else {
            // user said "no" in the prompt, so we don't do any initializing
        }

        // We don't want to read the default configuration file as config_raw. Since
        // it's empty there's no reason why the user would want to run it.

        return Ok(());
    };

    log::trace!("Using configuration file {}", config_file_path.display());

    let config = toml::from_str::<Configuration>(&config_raw).map_err(|err| {
        anyhow!("Could not parse `{root}/{CONFIG_FILE}` configuration file:\n{err}",)
    })?;

    let (remote_branch, commit_hash) = parse_if_maybe_hash(&config.remote_branch, " @ ");

    if config.repo.is_empty() {
        bail!(
            "You haven't specified a `repo` in your config, which can be for example:
  - `helix-editor/helix`
  - `microsoft/vscode`

  For more information see this guide: https://github.com/nik-rev/patchy/blob/main/README.md"
        );
    }

    let config_files = fs::read_dir(&config_path).map_err(|err| {
        anyhow!(
            "Failed to read files in directory `{}`:\n{err}",
            &config_path.display()
        )
    })?;

    let backed_up_files = {
        let mut backups = Vec::new();

        for config_file in config_files {
            let config_file = config_file?;

            let path = config_file.path();
            let backup = fs::read_to_string(&path)
                .map_err(|err| anyhow!("{err}"))
                .and_then(|contents| {
                    let filename = config_file.file_name();
                    let mut destination_backed_up =
                        tempfile::tempfile().map_err(|err| anyhow!("{err}"))?;

                    write!(destination_backed_up, "{contents}")?;

                    Ok((filename, destination_backed_up, contents))
                })
                .map_err(|err| {
                    anyhow!("Failed to create backups for configuration files:\n{err}")
                })?;

            backups.push(backup);
        }

        backups
    };

    let info = BranchAndRemote {
        branch: Branch {
            upstream_branch_name: remote_branch.clone(),
            local_branch_name: with_uuid(&remote_branch),
        },
        remote: Remote {
            repository_url: format!("https://github.com/{}.git", config.repo),
            local_remote_alias: with_uuid(&config.repo),
        },
    };

    git::add_remote_branch(&info, commit_hash.as_ref())?;

    let previous_branch = git::checkout_from_remote(
        &info.branch.local_branch_name,
        &info.remote.local_remote_alias,
    )?;

    let has_pull_requests = !config.pull_requests.is_empty();
    let has_branches = !config.branches.is_empty();

    if !has_pull_requests && !has_branches {
        log::warn!(
            "You haven't specified any pull requests or branches to fetch in your config, {}",
            display_link(
                "see the instructions on how to configure patchy.",
                "https://github.com/nik-rev/patchy?tab=readme-ov-file#config"
            )
        );
    } else {
        // Process pull requests
        if has_pull_requests {
            // TODO: make this concurrent, see https://users.rust-lang.org/t/processing-subprocesses-concurrently/79638/3
            // Git cannot handle multiple threads executing commands in the same repository,
            // so we can't use threads, but we can run processes in the background
            for pull_request in &config.pull_requests {
                let pull_request = ignore_octothorpe(pull_request);
                let (pull_request, commit_hash) = parse_if_maybe_hash(&pull_request, " @ ");
                // TODO: refactor this to not use such deep nesting
                match git::fetch_pull_request(
                    &config.repo,
                    &pull_request,
                    None,
                    commit_hash.as_ref(),
                )
                .await
                {
                    Ok((response, info)) => {
                        match git::merge_pull_request(
                            &info,
                            &pull_request,
                            &response.title,
                            &response.html_url,
                        ) {
                            Ok(()) => {
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
                            Err(err) => {
                                log::error!("{err}");
                            }
                        }
                    }
                    Err(err) => {
                        log::error!("Could not fetch branch from remote\n{err}");
                    }
                }
            }
        }

        // Process branches
        if has_branches {
            for branch_entry in &config.branches {
                let (branch_path, commit_hash) = parse_if_maybe_hash(branch_entry, " @ ");

                // Parse the branch path into owner/repo/branch format
                let parts: Vec<&str> = branch_path.split('/').collect();
                if parts.len() < 3 {
                    log::error!(
                        "Invalid branch format: {branch_path}. Expected format: owner/repo/branch"
                    );
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

                match git::fetch_branch(&remote, commit_hash.as_ref()).await {
                    Ok((_, info)) => {
                        match git::merge_into_main(
                            &info.branch.local_branch_name,
                            &info.branch.upstream_branch_name,
                        ) {
                            Ok(_) => {
                                log::info!(
                                    "Merged branch {}/{}/{} {}",
                                    owner.bright_blue(),
                                    repo.bright_blue(),
                                    branch_name.bright_blue(),
                                    commit_hash
                                        .map(|hash| format!(
                                            "at commit {}",
                                            hash.as_ref().bright_yellow()
                                        ))
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
                            Err(err) => {
                                log::error!("{err}");
                            }
                        }
                    }
                    Err(err) => {
                        log::error!("Could not fetch branch {owner}/{repo}/{branch_name}: {err}");
                    }
                }
            }
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

    for (file_name, _file, contents) in &backed_up_files {
        let path = GIT_ROOT.join(PathBuf::from(CONFIG_ROOT.as_str()).join(file_name));
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
    } else {
        let overwrite_command = format!(
            "git branch --move --force {temporary_branch} {}",
            config.local_branch
        );
        log::info!(
            "You can still manually overwrite {} with:\n  {overwrite_command}\n",
            config.local_branch.cyan(),
        );
        return Ok(());
    }

    Ok(())
}
