//! Main entry point to our Brainf*ck interpreter.
#![warn(missing_docs)]

use clap::Parser;

use bft_interp::BFVM;
use bft_types::BFprogram;

mod cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = cli::Opt::parse();
    let fname = opt.program;
    let src = BFprogram::from_file(fname)?;
    let vm: BFVM<u8> = BFVM::new(None, false);
    vm.interpret(&src);

    Ok(())
}
