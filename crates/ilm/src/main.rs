#![forbid(unknown_lints)]
#![deny(non_snake_case)]
#![deny(unreachable_patterns)]

use ilm_commands::command::run_command;
use ilm_core::base::process::read_process_use_with;
use std::process::ExitCode;

unsafe extern "C" {
    fn mi_process_info(elapsed_msecs: *mut usize, user_msecs: *mut usize, system_msecs: *mut usize, current_rss: *mut usize, peak_rss: *mut usize, current_commit: *mut usize, peak_commit: *mut usize, page_faults: *mut usize);
}

fn main() -> ExitCode {
    read_process_use_with(mi_process_info);
    run_command()
}

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;
