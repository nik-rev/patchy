use colored::Colorize as _;
use documented::Documented as _;

use crate::APP_NAME;
use crate::cli::branch_fetch::BranchFetch;
use crate::cli::gen_patch::GenPatch;
use crate::cli::init::Init;
use crate::cli::pr_fetch::PrFetch;
use crate::cli::run::Run;
use crate::cli::{Cli, SubCommand as _, Subcommand};

fn format_subcommand(command: &str, description: &str) -> String {
    let command = command.bright_yellow();
    format!("{command}\n    {}", format_description(description))
}

pub fn format_description(description: &str) -> String {
    format!("{} {description}", "»".bright_black())
}

pub fn help(subcommand: Option<Subcommand>) -> String {
    let author = "Nikita Revenco ".italic();
    let less_than = "<".bright_black().italic();
    let email = "pm@nikrev.com".italic();
    let greater_than = ">".bright_black().italic();
    let app_name = APP_NAME.bright_blue();
    let flags_label = "[<flags>]".bright_magenta();
    let command_str = "<command>".bright_yellow();
    let args = "[<args>]".bright_green();
    let version = env!("CARGO_PKG_VERSION");

    let help_and_version = format!(
        "    {}

    {}",
        Cli::HELP_FLAG,
        Cli::VERSION_FLAG,
    );

    let header = format!(
        "  {app_name} {version}
  {author}{less_than}{email}{greater_than}"
    );

    subcommand.map_or_else(
        // no specific subcommand supplied
        || {
            // main help menu
            let init = format_subcommand(Init::NAME, Init::DOCS);
            let run = format_subcommand(Run::NAME, Run::DOCS);
            let gen_patch = format_subcommand(GenPatch::NAME, GenPatch::DOCS);
            let pr_fetch = format_subcommand(PrFetch::NAME, PrFetch::DOCS);
            let branch_fetch = format_subcommand(BranchFetch::NAME, BranchFetch::DOCS);

            format!(
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

{help_and_version}

"
            )
        },
        |subcommand| {
            match subcommand {
                Subcommand::Init(_) => {
                    let cmd_name = Init::NAME;
                    let this_command_name = format!("{app_name} {}", cmd_name.bright_yellow());
                    let description = format_description(Init::DOCS);

                    format!(
                        "
{header}
        
  Usage:

    {this_command_name}
    {description}

  Flags:

{help_and_version}

"
                    )
                }
                Subcommand::Run(_) => {
                    let cmd_name = Run::NAME;
                    let this_command_name = format!("{app_name} {}", cmd_name.bright_yellow());
                    let description = format_description(Run::DOCS);
                    let yes_flag = Run::YES_FLAG;

                    format!(
                        "
{header}
        
  Usage:

    {this_command_name}
    {description}

  Flags:

{help_and_version}

    {yes_flag}
"
                    )
                }
                Subcommand::GenPatch(_) => {
                    let cmd_name = GenPatch::NAME;
                    let this_command_name = format!("{app_name} {}", cmd_name.bright_yellow());
                    let description = format_description(GenPatch::DOCS);
                    let patch_name_flag = GenPatch::PATCH_NAME_FLAG;

                    let example_1 = format!(
                        "{}
    {}",
                        "133cbaae83f710b793c98018cea697a04479bbe4".bright_green(),
                        format_description("Generate a single .patch file from one commit hash")
                    );

                    let example_2 = format!(
                        "{}
    {}",
                        "133cbaae83f710b793c98018cea697a04479bbe4 \
                         9ad5aa637ccf363b5d6713f66d0c2830736c35a9 \
                         cc75a895f344cf2fe83eaf6d78dfb7aeac8b33a4"
                            .bright_green(),
                        format_description(
                            "Generate several .patch files from several commit hashes"
                        )
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
                            "Generate several .patch files from several commit hashes and give 2 \
                             of them custom names"
                        )
                    );

                    format!(
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

    {patch_name_flag}

{help_and_version}
"
                    )
                }
                Subcommand::PrFetch(_) => {
                    let cmd_name = PrFetch::NAME;
                    let description = format_description(PrFetch::DOCS);

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
                            "Fetch several pull requests and choose custom branch names for the \
                             pull requests #10000 and #9191"
                        )
                    );

                    let example_4 = format!(
                        "{} {} {}
    {}",
                        "--repo-name=helix-editor/helix".bright_magenta(),
                        "11745 10000 9191 600".bright_green(),
                        "--checkout".bright_magenta(),
                        // NOTE: using concat for this because rustfmt breaks for some reason
                        format_description(concat!(
                            "Fetch several pull requests,",
                            " checkout the first one and use a custom github",
                            " repo: https://github.com/helix-editor/helix"
                        ))
                    );

                    let example_5 = format!(
                        "{}
    {}",
                        "11745 10000@be8f264327f6ae729a0b372ef01f6fde49a78310 9191 \
                         600@5d10fa5beb917a0dbe0ef8441d14b3d0dd15227b"
                            .bright_green(),
                        format_description("Fetch several pull requests at a certain commit")
                    );

                    let this_command_name = format!("{app_name} {}", cmd_name.bright_yellow());

                    let branch_name_flag = PrFetch::BRANCH_NAME_FLAG;

                    let checkout_flag = PrFetch::CHECKOUT_FLAG;

                    let repo_name_flag = PrFetch::REPO_NAME_FLAG;

                    format!(
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

    {branch_name_flag}

    {checkout_flag}

    {repo_name_flag}

{help_and_version}
"
                    )
                }
                Subcommand::BranchFetch(_) => {
                    let cmd_name = BranchFetch::NAME;
                    let description =
                        format_description("Fetch remote branches into a local branch");

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

                    format!(
                        "
{header}
        
  Usage:

    {this_command_name} {args} {flags_label}
    {description}

  Examples:

    {this_command_name} {example_1}

    {this_command_name} {example_2}

  Flags:

{help_and_version}
"
                    )
                }
            }
        },
    )
}
