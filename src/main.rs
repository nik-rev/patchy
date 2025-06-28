//! Patchy

use std::process::ExitCode;

use clap::Parser as _;

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();

    if let Err(err) = patchy::Cli::parse().command.execute().await {
        log::error!("{err}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
