//! Types for a Brainf*ck interpreter.

#![warn(missing_docs)]
use std::convert::AsRef;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::read;
use std::io;
use std::path::{Path, PathBuf};

/// Raw bytecodes for the brainf*ck VM.
#[derive(Debug)]
pub enum Instruction {
    /// Move the tape left one step.
    MoveLeft,

    /// Move the tape right one step.
    MoveRight,

    /// Increment the value at the current position of the tape.
    Increment,

    /// Decrement the value at the current position of the tape.
    Decrement,

    /// Receive one byte of data and store it in the current position of the tape.
    Input,

    /// Output the data at the current position of the tape.
    Output,

    /// If the value at the current position of the tape is zero, jump forward to the matching end
    /// loop.
    BeginLoop,

    /// If the value at the current position of the tape is not zero, jump backward to the matching
    /// begin loop.
    EndLoop,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::MoveLeft => write!(f, "Move left one location"),
            Self::MoveRight => write!(f, "Move right one location"),
            Self::Increment => write!(f, "Increment current location"),
            Self::Decrement => write!(f, "Decrement current location"),
            Self::Input => write!(f, "Accept one byte of input"),
            Self::Output => write!(f, "Output the current byte"),
            Self::BeginLoop => write!(f, "Start looping"),
            Self::EndLoop => write!(f, "Finish looping"),
        }
    }
}

impl Instruction {
    fn from_byte(c: u8) -> Option<Self> {
        match c {
            b'<' => Some(Instruction::MoveLeft),
            b'>' => Some(Instruction::MoveRight),
            b'+' => Some(Instruction::Increment),
            b'-' => Some(Instruction::Decrement),
            b',' => Some(Instruction::Input),
            b'.' => Some(Instruction::Output),
            b'[' => Some(Instruction::BeginLoop),
            b']' => Some(Instruction::EndLoop),
            _ => None,
        }
    }
}

/// Annotated bytecode instructions for brainf*ck.
#[derive(Debug)]
pub struct InputInstruction {
    inst: Instruction,
    line_number: usize,
    char_number: usize,
}

impl InputInstruction {
    /// A string representation of the instruction's location in the file.
    pub fn location(&self) -> String {
        format!("{}:{}", self.line_number, self.char_number)
    }

    /// Extract the underlying instruction.
    pub fn instruction(&self) -> &Instruction {
        &self.inst
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location() {
        let inst = InputInstruction {
            inst: Instruction::Increment,
            line_number: 100,
            char_number: 42,
        };
        assert_eq!(inst.location(), "100:42");
    }
}

/// A container to hold an entire Brainf*ck program.
#[derive(Debug)]
pub struct BFprogram {
    source_name: PathBuf,
    src: Vec<InputInstruction>,
}

impl BFprogram {
    /// Load data from a source file, and parse it into bytecode.
    ///
    /// # Errors
    /// This function will return an error if opening the file fails, or if there is an error
    /// reading the bytes within the file.
    pub fn from_file<P: AsRef<Path>>(file_name: P) -> io::Result<Self> {
        let data = read(&file_name)?;
        Ok(Self::new(file_name, data))
    }

    /// Parse Extended ASCII text into Brainf*ck bytecode. The Path `source_name` is used to store
    /// the name of the source of the text.
    /// ```
    /// use bft_types::BFprogram;
    /// let code = Vec::from(" <  > [\n]");
    /// let program = BFprogram::new("doc.test", code);
    ///
    /// assert_eq!(program.instructions().len(), 4);
    ///
    /// let mut iter = program.instructions().into_iter();
    /// let mut inst = iter.next();
    /// assert_eq!(inst.map_or(String::from("No Instruction"), |i| format!("{}", i.instruction())), "Move left one location");
    /// assert_eq!(inst.map_or(String::from("No Instruction"), |i| i.location()), "1:2");
    ///
    /// inst = iter.next();
    /// assert_eq!(inst.map_or(String::from("No Instruction"), |i| format!("{}", i.instruction())), "Move right one location");
    /// assert_eq!(inst.map_or(String::from("No Instruction"), |i| i.location()), "1:5");
    ///
    /// inst = iter.next();
    /// assert_eq!(inst.map_or(String::from("No Instruction"), |i| format!("{}", i.instruction())), "Start looping");
    /// assert_eq!(inst.map_or(String::from("No Instruction"), |i| i.location()), "1:7");
    ///
    /// inst = iter.next();
    /// assert_eq!(inst.map_or(String::from("No Instruction"), |i| format!("{}", i.instruction())), "Finish looping");
    /// assert_eq!(inst.map_or(String::from("No Instruction"), |i| i.location()), "2:1");
    ///
    /// assert!(iter.next().is_none());
    /// ```
    pub fn new<P: AsRef<Path>>(source_name: P, data: Vec<u8>) -> BFprogram {
        let mut src = Vec::new();
        // Technically we should split on b'\n', b'\r\n', or '\r'.
        // b'\r\n' will leave a b'\r' at the end of the line, this will be consumed without issue.
        // b'\r' was only used as a line terminator by Macs, pre OS X. We'll assume that this won't
        // be an issue...
        for (line_number, line) in data.split(|c| *c == b'\n').enumerate() {
            for (char_number, c) in line.iter().enumerate() {
                if let Some(inst) = Instruction::from_byte(*c) {
                    src.push(InputInstruction {
                        inst,
                        line_number: line_number + 1,
                        char_number: char_number + 1,
                    });
                }
            }
        }

        BFprogram {
            source_name: PathBuf::from(source_name.as_ref()),
            src,
        }
    }

    /// `instructions` allows us to access the underlying bytecode instructions.
    #[must_use]
    pub fn instructions(&self) -> &[InputInstruction] {
        self.src.as_slice()
    }

    /// get the name of the source file for the program.
    pub fn source(&self) -> &PathBuf {
        &self.source_name
    }
}
