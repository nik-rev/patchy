use colored::Colorize as _;
use patchy::cli::{self, Cli, Subcommand};
use patchy::commands::branch_fetch::branch_fetch;
use patchy::commands::help::{HELP_FLAG, VERSION_FLAG};
use patchy::commands::{gen_patch, help, init, pr_fetch, run};
use patchy::fail;
use reqwest::ClientBuilder;
use std::error::Error;
use std::{env, process};

use patchy::types::CommandArgs;

async fn process_subcommand(subcommand: &str, args: CommandArgs) -> Result<(), Box<dyn Error>> {
    match subcommand {
        // main commands
        "init" => init(&args)?,
        "run" => run(&args).await?,
        "gen-patch" => gen_patch(&args)?,
        // lower level commands
        "pr-fetch" => pr_fetch(&args).await?,
        "branch-fetch" => branch_fetch(&args).await?,
        unrecognized => {
            if !unrecognized.is_empty() {
                fail!(
                    "{}",
                    format!(
                        "  Unknown {unknown}: {}",
                        unrecognized,
                        unknown = if unrecognized.starts_with('-') {
                            "flag".bright_red()
                        } else {
                            "command".bright_red()
                        }
                    )
                    .bright_red()
                );
            }

            help(None)?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let args = cli::Cli::parse()?;

    // if args.help || args.subcommand.is_none() {
    //     todo!()
    // } else if args.version {
    //     print!("{}", env!("CARGO_PKG_VERSION"))
    // } else {
    //     match args.subcommand.unwrap() {
    //         Subcommand::Init => (),
    //     }
    // }

    // Ok(())

    let mut args = env::args();
    let _command_name = args.next();
    let subcommand = args.next().unwrap_or_default();

    let mut args: CommandArgs = args.collect();

    if subcommand.starts_with('-') {
        // We're not using any command, only flags
        args.insert(subcommand.clone());
    }

    if HELP_FLAG.is_in(&args) {
        help(Some(&subcommand))
    } else if VERSION_FLAG.is_in(&args) {
        print!("{}", env!("CARGO_PKG_VERSION"));

        Ok(())
    } else {
        match process_subcommand(subcommand.as_str(), args).await {
            Ok(()) => Ok(()),
            Err(msg) => {
                fail!("{msg}");
                process::exit(1);
            }
        }
    }
}
