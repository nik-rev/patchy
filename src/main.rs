//! Patchy

use std::process::ExitCode;

use clap::Parser as _;

#[tokio::main]
async fn main() -> ExitCode {
    let args = patchy::Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbosity.into())
        .init();

    if let Err(err) = args.command.execute(args.use_gh_cli).await {
        log::error!("{err}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
