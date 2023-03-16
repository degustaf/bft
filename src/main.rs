//! Main entry point to our Brainf*ck interpreter.
#![warn(missing_docs)]

use clap::Parser;
use std::process::ExitCode;

use bft_interp::BFVM;
use bft_types::BFprogram;

mod cli;

fn run_bft(options: &cli::Opt) -> Result<(), Box<dyn std::error::Error>> {
    let mut src = BFprogram::from_file(options.program.clone())?;
    src.validate_brackets()?;
    let vm: BFVM<u8> = BFVM::new(None, false);
    vm.interpret(&src);

    Ok(())
}

fn main() -> ExitCode {
    let opt = cli::Opt::parse();
    if let Err(error) = run_bft(&opt) {
        const BIN_NAME: &str = env!("CARGO_PKG_NAME");
        eprintln!("{BIN_NAME}: {error}");
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}
