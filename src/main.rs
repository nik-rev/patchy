use std::process::ExitCode;

use clap::Parser as _;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = patchy::Cli::parse().command.execute().await {
        patchy::fail!("{err}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
