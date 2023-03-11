//! The Virtual Machine that will run our Brainf*ck program.

#![warn(missing_docs)]

use bft_types::BFprogram;

/// Object that contains the internal workings of our Brainf*ck interpreter.
#[allow(dead_code)]
#[derive(Debug)]
pub struct BFVM<C: Default> {
    tape: Vec<C>,
    head: usize,
    growable: bool,
}

impl<C: Default> BFVM<C> {
    /// Construct a new VM with clean internal state.
    ///
    /// `capcity` specifies the size of the interal tape to use. A `capacity` of 0 indicates that a
    /// tape with the default capacity should be generated. `growable` is a flag to specifiy if the tape is gowable.
    pub fn new(capacity: usize, growable: bool) -> BFVM<C> {
        let c = if capacity == 0 { 30000 } else { capacity };
        let mut tape = Vec::new();
        tape.resize_with(c, C::default);
        BFVM {
            tape,
            head: 0,
            growable,
        }
    }

    /// The main interpreter that takes a program and (eventually) interprets it.
    pub fn interpret(&self, code: &BFprogram) {
        for inst in code.as_ref() {
            println!("{inst}");
        }
    }
}
