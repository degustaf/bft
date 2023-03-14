//! Main entry point to our Brainf*ck interpreter.

#![warn(missing_docs)]

use std::env::args_os;
use std::path::PathBuf;

use bft_interp::BFVM;
use bft_types::BFprogram;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fname = PathBuf::from(
        args_os()
            .nth(1)
            .ok_or("A file name to process is required.")?,
    );
    let src = BFprogram::from_file(fname)?;
    let vm: BFVM<u8> = BFVM::new(None, false);
    vm.interpret(&src);

    Ok(())
}
