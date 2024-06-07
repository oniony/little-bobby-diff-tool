use std::process::ExitCode;
use clap::{arg, Parser};
use postgres::Error;
use crate::{compare, db};
use crate::compare::report::Thing;
use crate::compare::report::ReportEntry::{Addition, Change, Match, Removal};
use crate::compare::report::Thing::{Column, TableConstraint, Property, Routine, Schema, Sequence, Table, Trigger, View};

pub struct CLI {}

impl CLI {
    pub fn run() -> Result<ExitCode, Error> {
        let args = Args::parse();

        let left_db = db::Database::connect(args.left.as_str())?;
        let right_db = db::Database::connect(args.right.as_str())?;

        let mut comparer = compare::Comparer::new(left_db, right_db, args.ignore_whitespace, args.ignore_column_ordinal);
        
        let mut differences = false;
        
        for schema in args.schema {
            let report = comparer.compare(schema.as_str())?;
            let report_differences = report.differences();
            
            if !report_differences.is_empty() {
                for entry in report_differences.iter() {
                    match entry {
                        Addition { path, thing: Column(name) } => println!("{}: column '{}' added", CLI::render_path(path), name),
                        Addition { path, thing: TableConstraint(name) } => println!("{}: constraint '{}' added", CLI::render_path(path), name),
                        Addition { path, thing: Property(name) } => println!("{}: property '{}' added", CLI::render_path(path), name),
                        Addition { path, thing: Routine(name) } => println!("{}: routine '{}' added", CLI::render_path(path), name),
                        Addition { path, thing: Schema(name) } => println!("{}: schema '{}' added", CLI::render_path(path), name),
                        Addition { path, thing: Sequence(name) } => println!("{}: sequence '{}' added", CLI::render_path(path), name),
                        Addition { path, thing: Table(name) } => println!("{}: table '{}' added", CLI::render_path(path), name),
                        Addition { path, thing: Trigger(name) } => println!("{}: trigger '{}' added", CLI::render_path(path), name),
                        Addition { path, thing: View(name) } => println!("{}: view '{}' added", CLI::render_path(path), name),
                        Removal { path, thing: Column(name) } => println!("{}: routine '{}' removed", CLI::render_path(path), name),
                        Removal { path, thing: TableConstraint(name) } => println!("{}: constraint '{}' removed", CLI::render_path(path), name),
                        Removal { path, thing: Property(name) } => println!("{}: property '{}' removed", CLI::render_path(path), name),
                        Removal { path, thing: Routine(name) } => println!("{}: routine '{}' removed", CLI::render_path(path), name),
                        Removal { path, thing: Schema(name) } => println!("{}: schema '{}' removed", CLI::render_path(path), name),
                        Removal { path, thing: Sequence(name) } => println!("{}: sequence '{}' removed", CLI::render_path(path), name),
                        Removal { path, thing: Table(name) } => println!("{}: table '{}' removed", CLI::render_path(path), name),
                        Removal { path, thing: Trigger(name) } => println!("{}: trigger '{}' removed", CLI::render_path(path), name),
                        Removal { path, thing: View(name) } => println!("{}: view '{}' removed", CLI::render_path(path), name),
                        Change { path, left_value, right_value } => println!("{}: changed from '{}' to '{}'", CLI::render_path(path), left_value, right_value),
                        Match { .. } => (),
                    }
                }
                differences = true;                
            }
        }
        
        let exit_code = match differences {
            true => ExitCode::FAILURE,
            false => ExitCode::SUCCESS,
        };

        return Ok(exit_code);
    }
    
    //TODO this can be moved once we have a dedicated type for the path
    pub fn render_path(path: &Vec<Thing>) -> String {
        path.iter().map(|t| t.to_string()).collect::<Vec<String>>().join(": ")
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, short = 'l', help = "The left database URL")]
    left: String,
    
    #[arg(short, long, short = 'r', help = "The right database URL")]
    right: String,
    
    #[arg(short, long, required = true, short = 's', help = "Schema to compare")]
    schema: Vec<String>,
    
    #[arg(short, long, short = 'w', help = "Ignore routine whitespace differences")]
    ignore_whitespace: bool,
    
    #[arg(short, long, short = 'o', help = "Ignore column ordering differences")]
    ignore_column_ordinal: bool,
}
