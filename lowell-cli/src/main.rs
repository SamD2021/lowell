use anyhow::Result;

mod cli;
mod tracing_init;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    tracing_init::init(&cli.global)?;
    cli.run()
}
