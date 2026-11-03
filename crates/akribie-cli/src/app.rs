//! Despacho dos verbos e tradução de erro em exit code.

mod emit;

use self::emit::batch::{emit_batch_check, emit_batch_render};
use self::emit::{
    Mode, emit_diff, emit_fmt, emit_import, emit_outcome, emit_render, emit_text, fail,
};

use crate::adapters::{StdFs, StdGit};
use crate::cli::{
    CheckArgs, Cli, Command, DiffArgs, FmtArgs, ImportArgs, InitArgs, PrimeArgs, RenderArgs,
};
use crate::commands;
use crate::output;
use std::process::ExitCode;

pub(crate) fn run(cli: Cli) -> ExitCode {
    match cli.command {
        Command::Check(args) => check(&args),
        Command::Fmt(args) => fmt(&args),
        Command::Schema => schema(),
        Command::Render(args) => render(&args),
        Command::Diff(args) => diff(&args),
        Command::Import(args) => import(&args),
        Command::Init(args) => init(&args),
        Command::Prime(args) => prime(&args),
    }
}
fn import(args: &ImportArgs) -> ExitCode {
    let fs = StdFs;
    let mode = if args.json { Mode::Json } else { Mode::Text };
    match commands::import::execute(args, &fs) {
        Ok(outcome) => emit_import(&outcome, mode),
        Err(error) => fail(mode, "import", &error),
    }
}
fn check_batch(args: &CheckArgs) -> ExitCode {
    let fs = StdFs;
    let mode = if args.json { Mode::Json } else { Mode::Text };
    match commands::check::batch::execute(args, &fs) {
        Ok(outcomes) => emit_batch_check(&outcomes, mode),
        Err(error) => fail(mode, "check", &error),
    }
}
fn render_batch(args: &RenderArgs) -> ExitCode {
    let fs = StdFs;
    let mode = if args.json { Mode::Json } else { Mode::Text };
    match commands::render::batch::execute(args, &fs, &StdGit) {
        Ok(outcomes) => emit_batch_render(&outcomes, mode),
        Err(error) => fail(mode, "render", &error),
    }
}
fn diff(args: &DiffArgs) -> ExitCode {
    let fs = StdFs;
    let mode = if args.json { Mode::Json } else { Mode::Text };
    match commands::diff::execute(args, &fs, &StdGit) {
        Ok(outcome) => emit_diff(&outcome, mode)
            .map_or_else(|error| fail(mode, "diff", &error), ExitCode::from),
        Err(error) => fail(mode, "diff", &error),
    }
}
fn check(args: &CheckArgs) -> ExitCode {
    if args.batch.is_some() {
        return check_batch(args);
    }
    let fs = StdFs;
    let mode = if args.json { Mode::Json } else { Mode::Text };
    match commands::check::execute(args, &fs) {
        Ok(outcome) => emit_outcome(&outcome, mode)
            .map_or_else(|error| fail(mode, "check", &error), ExitCode::from),
        Err(error) => fail(mode, "check", &error),
    }
}
fn render(args: &RenderArgs) -> ExitCode {
    let fs = StdFs;
    if args.batch.is_some() {
        return render_batch(args);
    }
    let mode = if args.json { Mode::Json } else { Mode::Text };
    match commands::render::execute(args, &fs, &StdGit) {
        Ok(outcome) => emit_render(&outcome, mode)
            .map_or_else(|error| fail(mode, "render", &error), ExitCode::from),
        Err(error) => fail(mode, "render", &error),
    }
}
fn fmt(args: &FmtArgs) -> ExitCode {
    let fs = StdFs;
    match commands::fmt::execute(args, &fs) {
        Ok(outcome) => emit_fmt(&outcome),
        Err(error) => fail(Mode::Text, "fmt", &error),
    }
}
fn schema() -> ExitCode {
    match output::write_stdout(&commands::schema::render()) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => fail(Mode::Text, "schema", &error),
    }
}

fn init(args: &InitArgs) -> ExitCode {
    let fs = StdFs;
    let force = if args.force {
        commands::init::Force::Yes
    } else {
        commands::init::Force::No
    };
    match commands::init::execute(force, &fs) {
        Ok(outcome) => emit_text(&outcome.render(), 0, "init"),
        Err(error) => fail(Mode::Text, "init", &error),
    }
}

fn prime(args: &PrimeArgs) -> ExitCode {
    let length = if args.long {
        commands::prime::Length::Long
    } else {
        commands::prime::Length::Compact
    };
    emit_text(&commands::prime::render(length), 0, "prime")
}
