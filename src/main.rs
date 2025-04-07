use core::error;
use std::env;
use std::process::ExitCode;

use patchy::cli::flags::HelpOrVersion;
use patchy::cli::{Cli, Subcommand};
use patchy::{PatchyError, commands};

async fn main_impl() -> Result<Option<String>, Box<dyn error::Error>> {
    let args = Cli::parse().map_err(PatchyError::CliParseError)?;

    let subcommand = match args.help_or_version {
        HelpOrVersion::Help => {
            return Ok(Some(commands::help(args.subcommand)));
        },
        HelpOrVersion::Version => {
            return Ok(Some(env!("CARGO_PKG_VERSION").to_owned()));
        },
        HelpOrVersion::None if args.subcommand.is_none() => {
            return Ok(Some(commands::help(args.subcommand)));
        },
        HelpOrVersion::None => args
            .subcommand
            .expect("checked that we DO have a subcommand in an earlier branch"),
    };

    match subcommand {
        Subcommand::Init(_init_args) => commands::init(),
        Subcommand::Run(run_args) => Ok(commands::run(run_args).await?),
        Subcommand::GenPatch(gen_patch_args) => commands::gen_patch(gen_patch_args),
        Subcommand::PrFetch(pr_fetch_args) => Ok(commands::pr_fetch(pr_fetch_args).await?),
        Subcommand::BranchFetch(branch_fetch_args) => {
            Ok(commands::branch_fetch(branch_fetch_args).await?)
        },
    }?;

    Ok(None)
}

#[tokio::main]
async fn main() -> ExitCode {
    match main_impl().await {
        Ok(msg) => {
            println!("{}", msg.unwrap_or_default());
            ExitCode::SUCCESS
        },
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        },
    }
}
