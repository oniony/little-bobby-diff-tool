mod db;
mod compare;
mod cli;

use std::process::ExitCode;
use postgres::Error;
use crate::cli::CLI;

fn main() -> Result<ExitCode, Error> {
    let same = CLI::run();
    
    same
}

