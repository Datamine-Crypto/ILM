#![forbid(unknown_lints)]
#![deny(non_snake_case)]
#![deny(unreachable_patterns)]

use ilm_commands::command::run_command;
use std::process::ExitCode;

fn main() -> ExitCode {
    run_command()
}

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;
