use std::process::ExitCode;
use clap::Parser;
use postgres::Error;
use crate::{compare, db};

pub struct CLI {}

impl CLI {
    pub fn run() -> Result<ExitCode, Error> {
        let args = Args::parse();

        let left_db = db::Database::connect(args.left_database_url.as_str())?;
        let right_db = db::Database::connect(args.right_database_url.as_str())?;

        let mut comparer = compare::Comparer::new(left_db, right_db);
        let same = comparer.compare(args.schema.as_str())?;
        
        let exit_code = match same {
            true => ExitCode::SUCCESS,
            false => ExitCode::FAILURE,
        };

        return Ok(exit_code);
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    left_database_url: String,
    
    #[arg(short, long)]
    right_database_url: String,
    
    #[arg(short, long)]
    schema: String,
}
