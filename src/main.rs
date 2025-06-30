//! Patchy

use std::io::Write as _;
use std::process::ExitCode;

use clap::{
    Parser as _,
    builder::styling::{AnsiColor, Reset},
};
use log::Level;

#[tokio::main]
async fn main() -> ExitCode {
    let args = patchy::Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbosity.into())
        .format(|buf, record| {
            let color = match record.level() {
                Level::Error => AnsiColor::BrightRed,
                Level::Warn => AnsiColor::BrightYellow,
                Level::Info => AnsiColor::BrightGreen,
                Level::Debug => AnsiColor::BrightBlue,
                Level::Trace => AnsiColor::BrightCyan,
            }
            .on_default()
            .render();
            let black = AnsiColor::BrightBlack.render_fg();
            let level = record.level();
            let message = record.args();

            writeln!(buf, "{black}[{color}{level}{black}]{Reset} {message}",)
        })
        .init();

    if let Err(err) = args.command.execute(args.use_gh_cli).await {
        log::error!("{err}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
