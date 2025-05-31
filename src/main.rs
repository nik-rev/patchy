use clap::Parser as _;

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = patchy::Cli::parse().command.execute().await {
        patchy::fail!("{err}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
