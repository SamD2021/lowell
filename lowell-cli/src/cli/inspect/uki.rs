use anyhow::Result;
use clap::{Args, ValueEnum};
use lowell_core::inspect::uki::{self, Report, UkiOptions};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Output {
    Human,
    Json,
    JsonPretty,
}

#[derive(Args, Debug)]
pub struct UkiArgs {
    /// Path to the UKI to inspect
    #[arg(long)]
    file: PathBuf,
    /// Output format (human by default)
    #[arg(long, value_enum, default_value_t = Output::Human)]
    format: Output,
    /// Show more fields in human output
    #[arg(long, short = 'v')]
    verbose: bool,
}

impl UkiArgs {
    pub fn run(self) -> Result<()> {
        let report = uki::inspect(UkiOptions { file: self.file })?;
        match self.format {
            Output::Human => print_human(&report, self.verbose)?,
            Output::Json => {
                serde_json::to_writer(io::stdout(), &report)?;
                io::stdout().write_all(b"\n")?;
            }
            Output::JsonPretty => {
                serde_json::to_writer_pretty(io::stdout(), &report)?;
                io::stdout().write_all(b"\n")?;
            }
        }
        Ok(())
    }
}

fn print_human(r: &Report, verbose: bool) -> Result<()> {
    let mut out = io::BufWriter::new(io::stdout());

    // Header / identity
    writeln!(
        out,
        "{} • {} • {}",
        r.os_release
            .as_ref()
            .and_then(|o| o.name.as_deref())
            .unwrap_or("<unknown>"),
        r.arch,
        if r.pe32_plus { "PE32+" } else { "PE32" }
    )?;

    // Secure Boot / signatures
    let sig = if r.has_signature {
        format!("signed ({} certs)", r.cert_count)
    } else {
        "unsigned".to_string()
    };
    writeln!(out, "secure-boot: {sig}")?;

    // Cmdline (trimmed already)
    if !r.cmdline.is_empty() {
        writeln!(out, "cmdline: {}", r.cmdline)?;
    }

    // Sections
    writeln!(
        out,
        "kernel  : {} ({})",
        fmt_bytes(r.linux.size),
        fmt_offset(r.linux.offset)
    )?;
    if verbose {
        writeln!(out, "  sha256: {}", r.linux.sha256)?;
    }

    writeln!(
        out,
        "initrd  : {} ({}), compression: {}",
        fmt_bytes(r.initrd.section.size),
        fmt_offset(r.initrd.section.offset),
        r.initrd.compression
    )?;
    if verbose {
        writeln!(out, "  sha256: {}", r.initrd.section.sha256)?;
    }

    out.flush()?;
    Ok(())
}

// tiny helpers (no deps)
fn fmt_bytes(n: usize) -> String {
    // MiB with one decimal place
    let mib = (n as f64) / (1024.0 * 1024.0);
    format!("{mib:.1} MiB")
}
fn fmt_offset(off: usize) -> String {
    format!("offset {off:#x}")
}
