mod db;
mod compare;

use std::process::ExitCode;

fn main() -> ExitCode {
    //TODO urls from arguments
    let left_db = db::Database::connect("postgresql://postgres:postgres@localhost:8901/postgres").unwrap();
    let right_db = db::Database::connect("postgresql://postgres:postgres@localhost:8902/postgres").unwrap();
    
    let mut comparer = compare::Comparer::new(left_db, right_db);
    
    let same = comparer.compare();
    
    match same {
        true => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}

