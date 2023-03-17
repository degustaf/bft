//! The Virtual Machine that will run our Brainf*ck program.

#![warn(missing_docs)]

use std::num::NonZeroUsize;

use bft_types::BFprogram;

/// Brainf*ck interpreter internal state.
#[allow(dead_code)]
#[derive(Debug)]
pub struct BFVM<C> {
    /// Block of memory for the program to work on.
    tape: Vec<C>,

    /// Index of where the program is pointing to in the tape.
    head: usize,

    /// When true, the VM is allowed to grow the tape for additional space as needed.
    growable: bool,
}

impl<C: Default> BFVM<C> {
    /// Construct a new VM with clean internal state.
    ///
    /// `capcity` specifies the size of the interal tape to use. A `capacity` of 0 indicates that a
    /// tape with the default capacity should be generated. `growable` is a flag to specifiy if the tape is gowable.
    pub fn new(capacity: Option<NonZeroUsize>, growable: bool) -> BFVM<C> {
        let c = capacity.map_or(30000, NonZeroUsize::get);
        let mut tape = Vec::new();
        tape.resize_with(c, C::default);
        BFVM {
            tape,
            head: 0,
            growable,
        }
    }
}

impl<C> BFVM<C> {
    /// The main interpreter that takes a program and (eventually) interprets it.
    pub fn interpret(&self, code: &BFprogram) {
        for inst in code.instructions() {
            println!(
                "[{:?}:{}] {}",
                code.source(),
                inst.location(),
                inst.instruction()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_vm() {
        let mut vm: BFVM<u8> = BFVM::new(NonZeroUsize::new(200), false);
        assert_eq!(vm.tape.len(), 200);

        vm = BFVM::new(NonZeroUsize::new(0), false);
        assert_eq!(vm.tape.len(), 30000);
    }
}
