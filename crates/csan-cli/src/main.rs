#![forbid(unsafe_code)]

//! The `csan` command-line tool.
//!
//! Everything — parsing, execution, printing, exit-code policy — lives in
//! [`csan_cli::run_from_args`]; this shell only feeds it the process
//! argv.

use std::env;
use std::process::ExitCode;
// `unused_crate_dependencies` is checked per compilation target; these are
// used by the library half of this crate, just not by the binary shell.
use {clap as _, csan as _, serde_json as _};

fn main() -> ExitCode {
    ExitCode::from(csan_cli::run_from_args(env::args_os()))
}
