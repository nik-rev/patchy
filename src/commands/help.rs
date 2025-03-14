use colored::Colorize as _;

use crate::{
    APP_NAME,
    commands::{
        gen_patch::GEN_PATCH_NAME_FLAG,
        pr_fetch::{PR_FETCH_BRANCH_NAME_FLAG, PR_FETCH_CHECKOUT_FLAG, PR_FETCH_REPO_NAME_FLAG},
        run::RUN_YES_FLAG,
    },
    flags::Flag,
};

fn format_subcommand(command: &str, description: &str) -> String {
    let command = command.bright_yellow();
    format!("{command}\n    {}", format_description(description))
}

pub fn format_description(description: &str) -> String {
    format!("{} {description}", "»".bright_black())
}

pub static HELP_FLAG: Flag<'static> = Flag {
    short: "-h",
    long: "--help",
    description: "Print this message",
};

pub static VERBOSE_FLAG: Flag<'static> = Flag {
    short: "-V",
    long: "--verbose",
    description: "Increased logging information",
};

pub static VERSION_FLAG: Flag<'static> = Flag {
    short: "-v",
    long: "--version",
    description: "Get patchy version",
};

pub fn help(command: Option<&str>) -> anyhow::Result<()> {
    let author = "Nikita Revenco ".italic();
    let less_than = "<".bright_black().italic();
    let email = "pm@nikrev.com".italic();
    let greater_than = ">".bright_black().italic();
    let app_name = APP_NAME.bright_blue();
    let flags_label = "[<flags>]".bright_magenta();
    let command_str = "<command>".bright_yellow();
    let args = "[<args>]".bright_green();
    let version = env!("CARGO_PKG_VERSION");
    let init = format_subcommand("init", "Create example config file");
    let pr_fetch = format_subcommand(
        "pr-fetch",
        "Fetch pull request for a GitHub repository as a local branch",
    );
    let branch_fetch = format_subcommand(
        "branch-fetch",
        "Fetch branches for a GitHub repository as a local branch",
    );
    let gen_patch = format_subcommand("gen-patch", "Generate a .patch file from commit hashes");
    let run = format_subcommand("run", &format!("Start {APP_NAME}"));
    let header = format!(
        "  {app_name} {version}
  {author}{less_than}{email}{greater_than}"
    );
    match command {
        Some(cmd_name @ "init") => {
            let this_command_name = format!("{app_name} {}", cmd_name.bright_yellow());

            let description = format_description("Create example config file");

            println!(
                "
{header}
        
  Usage:

    {this_command_name}
    {description}

  Flags:

    {HELP_FLAG}
",
            );
        }
        Some(cmd_name @ "run") => {
            let this_command_name = format!("{app_name} {}", cmd_name.bright_yellow());

            let description = format_description("Create example config file");

            println!(
                "
{header}
        
  Usage:

    {this_command_name}
    {description}

  Flags:

    {HELP_FLAG}

    {RUN_YES_FLAG}
",
            );
        }
        Some(cmd_name @ "gen-patch") => {
            let this_command_name = format!("{app_name} {}", cmd_name.bright_yellow());

            let description = format_description("Generate a .patch file from commit hashes");

            let example_1 = format!(
                "{}
    {}",
                "133cbaae83f710b793c98018cea697a04479bbe4".bright_green(),
                format_description("Generate a single .patch file from one commit hash")
            );

            let example_2 = format!(
                "{}
    {}",
                "133cbaae83f710b793c98018cea697a04479bbe4 9ad5aa637ccf363b5d6713f66d0c2830736c35a9 cc75a895f344cf2fe83eaf6d78dfb7aeac8b33a4".bright_green(),
                format_description("Generate several .patch files from several commit hashes")
            );

            let example_3 = format!(
                "{} {} {} {} {}
    {}",
                "133cbaae83f710b793c98018cea697a04479bbe4".bright_green(),
                "--patch-filename=some-patch".bright_magenta(),
                "9ad5aa637ccf363b5d6713f66d0c2830736c35a9".bright_green(),
                "--patch-filename=another-patch".bright_magenta(),
                "cc75a895f344cf2fe83eaf6d78dfb7aeac8b33a4".bright_green(),
                format_description(
                    "Generate several .patch files from several commit hashes and give 2 of them custom names"
                )
            );

            println!(
                "
{header}
        
  Usage:

    {this_command_name}
    {description}

  Examples:

    {this_command_name} {example_1}

    {this_command_name} {example_2}

    {this_command_name} {example_3}

  Flags:

    {GEN_PATCH_NAME_FLAG}

    {HELP_FLAG}
",
            );
        }
        Some(cmd_name @ "branch-fetch") => {
            let description = format_description("Fetch remote branches into a local branch");

            let example_1 = format!(
                "{}
    {}",
                "helix-editor/helix/master".bright_green(),
                format_description("Fetch a single branch")
            );
            let example_2 = format!(
                "{}
    {}",
                "'helix-editor/helix/master@6049f20'".bright_green(),
                format_description("Fetch a single branch at a certain commit")
            );

            let this_command_name = format!("{app_name} {}", cmd_name.bright_yellow());

            println!(
                "
{header}
        
  Usage:

    {this_command_name} {args} {flags_label}
    {description}

  Examples:

    {this_command_name} {example_1}

    {this_command_name} {example_2}
",
            );
        }
        Some(cmd_name @ "pr-fetch") => {
            let description = format_description("Fetch pull requests into a local branch");

            let example_1 = format!(
                "{}
    {}",
                "11745".bright_green(),
                format_description("Fetch a single pull request")
            );

            let example_2 = format!(
                "{}
    {}",
                "11745 10000 9191 600".bright_green(),
                format_description("Fetch several pull requests")
            );

            let example_3 = format!(
                "{} {} {} {} {}
    {}",
                "11745 10000".bright_green(),
                "--branch-name=some-pr".bright_magenta(),
                "9191".bright_green(),
                "--branch-name=another-pr".bright_magenta(),
                "600".bright_green(),
                format_description(
                    "Fetch several pull requests and choose custom branch names for the pull requests #10000 and #9191"
                )
            );

            let example_4 = format!(
                "{} {} {}
    {}",
                "--repo-name=helix-editor/helix".bright_magenta(),
                "11745 10000 9191 600".bright_green(),
                "--checkout".bright_magenta(),
                format_description(
                    "Fetch several pull requests, checkout the first one and use a custom github repo: https://github.com/helix-editor/helix"
                )
            );

            let example_5 = format!(
                "{}
    {}",
                "11745 10000@be8f264327f6ae729a0b372ef01f6fde49a78310 9191 600@5d10fa5beb917a0dbe0ef8441d14b3d0dd15227b".bright_green(),
                format_description("Fetch several pull requests at a certain commit")
            );
            let this_command_name = format!("{app_name} {}", cmd_name.bright_yellow());

            println!(
                "
{header}
        
  Usage:

    {this_command_name} {args} {flags_label}
    {description}

  Examples:

    {this_command_name} {example_1}

    {this_command_name} {example_2}

    {this_command_name} {example_3}

    {this_command_name} {example_4}

    {this_command_name} {example_5}

  Flags:

    {PR_FETCH_BRANCH_NAME_FLAG}

    {PR_FETCH_CHECKOUT_FLAG}

    {PR_FETCH_REPO_NAME_FLAG}

    {HELP_FLAG}
",
            );
        }
        _ => {
            println!(
                "
{header}
        
  Usage:

    {app_name} {command_str} {args} {flags_label}

  Commands:

    {init} 

    {run}

    {gen_patch} 

    {pr_fetch} 

    {branch_fetch} 

  Flags:

    {HELP_FLAG}

    {VERSION_FLAG}
"
            );
        }
    }

    Ok(())
}
