use core::error;
use std::env;

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
        HelpOrVersion::None => args.subcommand.unwrap(),
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
async fn main() {
    match main_impl().await {
        Ok(ok) => {
            println!("{}", ok.unwrap_or_default());
            std::process::exit(0);
        },
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        },
    }
}
