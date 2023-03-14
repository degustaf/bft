#![warn(missing_docs)]

use clap::Parser;
use std::path::PathBuf;

/// A Brainf*ck interpreter.
#[derive(Debug, Parser)]
#[command(author, version, about, name = "bft")]
pub struct Opt {
    /// The Brainf*ck program to run.
    #[clap(required(true), value_parser)]
    pub program: PathBuf,

    /// Number of cells for the programs tape.
    #[arg(short, long, default_value_t = 30000, value_parser = clap::value_parser!(u64).range(1..))]
    pub cells: usize,

    /// Allow the program tape to be automatically extended.
    #[arg(short, long, default_value_t = false)]
    pub extensible: bool,
}
