use clap::Parser as _;

use std::process::ExitCode;

use patchy::{cli::Command, commands};

async fn main_impl() -> anyhow::Result<()> {
    let arg = patchy::cli::Args::parse();

    match arg.command {
        Command::Init => commands::init()?,
        Command::Run { yes } => commands::run(yes).await?,
        Command::GenPatch { commit, filename } => {
            commands::gen_patch(commit, filename)?;
        }
        Command::PrFetch {
            pr,
            remote,
            branch,
            commit,
            checkout,
        } => commands::pr_fetch(pr, remote, branch, commit, checkout).await?,
        Command::BranchFetch {
            remote,
            commit,
            checkout,
        } => commands::branch_fetch(remote, commit, checkout).await?,
    }

    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = main_impl().await {
        eprintln!("{err}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
