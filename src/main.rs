mod db;
mod compare;
mod cli;
mod string;

use std::process;
use std::process::ExitCode;
use postgres::Error;
use crate::cli::CLI;

fn main() -> Result<ExitCode, Error> {
    let cli = CLI::new();
    let result = cli.run();

    match result {
        Ok(count) => process::exit(count),
        _ => process::exit(-1),
    }
}

