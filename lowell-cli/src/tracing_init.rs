// SPDX-License-Identifier: MIT OR Apache-2.0
use anyhow::Result;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::cli::GlobalArgs;

pub fn init(g: &GlobalArgs) -> Result<()> {
    // If RUST_LOG is set, honor it entirely (user can set goblin=trace themselves).
    let filter = if std::env::var_os("RUST_LOG").is_some() {
        EnvFilter::from_default_env()
    } else {
        // Default to the CLI level, but quiet down goblin’s debug churn.
        EnvFilter::new(g.log_level.as_str()).add_directive("goblin=warn".parse().unwrap())
        // add more noisy deps here if needed:
        // .add_directive("goblin::pe=warn".parse().unwrap())
        // .add_directive("object=warn".parse().unwrap())
    };

    tracing_subscriber::registry()
        .with(fmt::layer().without_time())
        .with(filter)
        .init();

    Ok(())
}
