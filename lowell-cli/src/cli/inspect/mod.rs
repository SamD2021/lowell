// SPDX-License-Identifier: MIT OR Apache-2.0
mod uki;

use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct InspectArgs {
    #[command(subcommand)]
    cmd: InspectCmd,
}

#[derive(Subcommand, Debug)]
enum InspectCmd {
    /// Uki contents from a UKI
    Uki(uki::UkiArgs),
}

impl InspectArgs {
    pub fn run(self) -> Result<()> {
        match self.cmd {
            InspectCmd::Uki(a) => a.run(),
        }
    }
}
