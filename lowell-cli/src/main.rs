use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::ValueEnum;
use clap::{Args, Parser, Subcommand};
use lowell_core::profile::Profile;
use tracing::{Level, level_filters::LevelFilter};
use tracing::{debug, info};
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Subcommand)]
enum Cmd {
    Build(BuildArgs),
}

#[derive(Args, Debug)]
struct BuildArgs {
    /// Profile file (TOML)
    #[arg(long)]
    profile: PathBuf,
    /// Path to write initramfs
    #[arg(long)]
    out_initramfs: String,
    /// Optional UKI output
    #[arg(long)]
    uki_out: Option<String>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for Level {
    fn from(v: LogLevel) -> Self {
        match v {
            LogLevel::Error => Level::ERROR,
            LogLevel::Warn => Level::WARN,
            LogLevel::Info => Level::INFO,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Trace => Level::TRACE,
        }
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(v: LogLevel) -> Self {
        match v {
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Trace => LevelFilter::TRACE,
        }
    }
}

#[derive(Parser)]
#[command(name = "lowell", version, about = "Hermetic initramfs/UKI builder")]
struct Cli {
    /// Sets the log verbosity
    #[arg(long, value_enum, default_value_t = LogLevel::Info )]
    log_level: LogLevel,
    #[command(subcommand)]
    cmd: Cmd,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Build an EnvFilter that defaults to --log-level but still honors RUST_LOG if set.
    let filter =
        EnvFilter::from_default_env().add_directive(LevelFilter::from(cli.log_level).into());

    tracing_subscriber::registry()
        .with(fmt::layer().without_time())
        .with(filter)
        .init(); // sets the global subscriber
    //
    match cli.cmd {
        Cmd::Build(args) => run_build(args)?,
    }
    Ok(())
}

fn run_build(args: BuildArgs) -> Result<()> {
    info!("reading profile: {0:?}", args.profile);
    // Parse Profile
    let p = std::fs::read_to_string(&args.profile)
        .with_context(|| format!("failed to read: {}", args.profile.display()))?;
    let prof: Profile =
        toml::from_str(&p).with_context(|| format!("{0:?} is not valid TOML", &p))?;
    debug!(?prof, "parsed profile");

    // Build Initramfs
    let contents = "placeholder";
    std::fs::write(args.out_initramfs, contents)?;
    Ok(())
}
