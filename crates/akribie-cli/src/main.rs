//! Binário `akr`.
//!
//! A CLI é a única camada que monta as implementações reais das portas (`StdFs`, `StdGit`,
//! `StdEnv`, `StderrLogger`). O domínio em `akribie-core` não toca o sistema operacional.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod adapters;
mod app;
mod batch;
mod cli;
mod commands;
mod envelope;
mod input;
mod output;
mod params;

use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    match cli::Cli::try_parse_from(std::env::args_os()) {
        Ok(cli) => app::run(cli),
        Err(error) => {
            let code = error.exit_code();
            if error.print().is_err() {
                // stderr indisponível: resta o exit code.
            }
            ExitCode::from(u8::try_from(code).unwrap_or(2))
        }
    }
}
