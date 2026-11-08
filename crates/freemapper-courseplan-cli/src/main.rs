//! The `freemapper-courseplan` binary. Commands come with the milestones in the design
//! (`docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md`).

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "freemapper-courseplan",
    version,
    about = "Course setting for orienteering"
)]
struct Cli {}

fn main() {
    Cli::parse();
}
