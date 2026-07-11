//! `wickra-gym` — the reference rollout runner over `gym-core`.

mod args;
mod run;

use std::process::ExitCode;

use clap::Parser;

use crate::args::Args;

fn main() -> ExitCode {
    let args = Args::parse();
    match run::run(&args) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
