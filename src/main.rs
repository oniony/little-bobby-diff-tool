mod db;
mod compare;
mod cli;
mod string;

use std::process;
use crate::cli::CLI;

#[tokio::main]
async fn main() {
    let cli = CLI::new();
    let result = cli.run().await;

    match result {
         Ok(count) => process::exit(count),
         Err(err) => {
             eprintln!("{}", err);
             process::exit(-1);
         },
     }
}
