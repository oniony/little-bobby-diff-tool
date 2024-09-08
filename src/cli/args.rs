use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, short = 'l', help = "The left database URL")]
    pub left: String,

    #[arg(short, long, short = 'r', help = "The right database URL")]
    pub right: String,

    #[arg(short, long, required = true, short = 's', help = "Schema to compare")]
    pub schema: Vec<String>,

    #[arg(short, long, short = 'w', help = "Ignore routine whitespace differences")]
    pub ignore_whitespace: bool,

    #[arg(short, long, short = 'o', help = "Ignore column ordering differences")]
    pub ignore_column_ordinal: bool,

    #[arg(short, long, short = 'p', help = "Ignore privilege changes")]
    pub ignore_privileges: bool,

    #[arg(short, long, short = 'v', help = "Show matches")]
    pub verbose: bool,

    #[arg(short, long, short = 'c', alias = "colour", help = "Use colour", default_value = "auto")]
    pub color: Colouring,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Colouring {
    Auto,
    Always,
    Never,
}
