// SPDX-License-Identifier: MIT OR Apache-2.0
use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "lowell", version, about = "Hermetic initramfs/UKI builder")]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,
    #[command(subcommand)]
    cmd: Cmd,
}

impl Cli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
    pub fn run(self) -> Result<()> {
        match self.cmd {
            Cmd::Inspect(a) => a.run(),
        }
    }
}

#[derive(Args, Debug)]
pub struct GlobalArgs {
    /// Sets the log verbosity (overridden by RUST_LOG if set)
    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    Inspect(inspect::InspectArgs),
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        }
    }
}

mod inspect;

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::CommandFactory;
    #[test]
    fn cli_ok() {
        Cli::command().debug_assert();
    }
}
