/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

// We use a separate lib since doctests in a binary are ignored,
// and we'd like to use doctests.

#![forbid(unsafe_code)]

use std::process;

use clap::Parser;
use td_util::command::display_command;

/// Run `buck2 audit` with all the arguments required for BTD/Citadel.
#[derive(Parser, Debug)]
pub struct Args {
    #[clap(subcommand)]
    mode: AuditMode,
}

#[derive(Parser, Debug)]
enum AuditMode {
    /// Run `buck2 audit cell` with the right arguments.
    Cell(Common),
    /// Run `buck2 audit config` with the right arguments.
    Config(Common),
}

#[derive(Parser, Debug)]
struct Common {
    /// The command for running Buck
    #[arg(long, default_value = "buck2")]
    buck: String,

    #[arg(long)]
    dry_run: bool,
}

/// It doesn't matter which config we run cells in, they should all be the same,
/// so avoid invaliding the daemon.
const REUSE_CONFIG: &str = "--reuse-current-config";

pub fn audit_cell_arguments() -> &'static [&'static str] {
    &["audit", "cell", "--json", REUSE_CONFIG]
}

pub fn audit_config_arguments() -> &'static [&'static str] {
    &[
        "audit",
        "config",
        "--json",
        "--all-cells",
        "buildfile.name",
        "buildfile.name_v2",
        REUSE_CONFIG,
    ]
}

pub fn main(args: Args) -> anyhow::Result<()> {
    let (common, arguments) = match args.mode {
        AuditMode::Cell(common) => (common, audit_cell_arguments()),
        AuditMode::Config(common) => (common, audit_config_arguments()),
    };

    let mut command = std::process::Command::new(common.buck);
    command.args(arguments);

    if common.dry_run {
        println!("{}", display_command(&command));
        return Ok(());
    }

    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        process::exit(status.code().unwrap_or(1));
    }
}
