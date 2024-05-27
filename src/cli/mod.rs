use std::process::ExitCode;
use clap::{arg, Parser};
use postgres::Error;
use crate::{compare, db};

pub struct CLI {}

impl CLI {
    pub fn run() -> Result<ExitCode, Error> {
        let args = Args::parse();

        let left_db = db::Database::connect(args.left.as_str())?;
        let right_db = db::Database::connect(args.right.as_str())?;

        let mut comparer = compare::Comparer::new(left_db, right_db);
        
        let mut same = true;

        for schema in args.schema {
            let schema_same = comparer.compare(schema.as_str())?;
            same = same & schema_same;
        }
        
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
    left: String,
    
    #[arg(short, long)]
    right: String,
    
    #[arg(short, long)]
    schema: Vec<String>,
}
