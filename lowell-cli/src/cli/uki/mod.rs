// SPDX-License-Identifier: MIT OR Apache-2.0
mod inspect;

use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct UkiArgs {
    #[command(subcommand)]
    cmd: UkiCmd,
}

#[derive(Subcommand, Debug)]
enum UkiCmd {
    /// Inspect contents from a UKI
    Inspect(inspect::InspectArgs),
}

impl UkiArgs {
    pub fn run(self) -> Result<()> {
        match self.cmd {
            UkiCmd::Inspect(a) => a.run(),
        }
    }
}
