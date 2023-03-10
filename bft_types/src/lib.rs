//! Types for a Brainf*ck interpreter.

#![warn(missing_docs)]
use std::convert::AsRef;
use std::fmt;
use std::fmt::Result as fmtResult;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Error as ioError;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum Instruction {
    MoveLeft,
    MoveRight,
    Increment,
    Decrement,
    Input,
    Output,
    BeginLoop,
    EndLoop,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmtResult {
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
    source_name: PathBuf,
}

impl Display for InputInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}:{}:{}] {}",
            self.source_name.display(),
            self.line_number,
            self.char_number,
            self.inst
        )
    }
}

/// A container to hold an entire Brainf*ck program.
#[allow(dead_code)]
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
    pub fn from_file<P: AsRef<Path>>(file_name: P) -> Result<BFprogram, ioError> {
        let file = File::open(&file_name)?.bytes();
        let data: Vec<u8> = file.collect::<Result<Vec<u8>, ioError>>()?;
        Ok(BFprogram::new(file_name, data))
    }

    /// Parse Extended ASCII text into Brainf*ck bytecode. The Path `source_name` is used to store
    /// the name of the source of the text.
    pub fn new<P: AsRef<Path>, V: AsRef<Vec<u8>>>(source_name: P, data: V) -> BFprogram {
        let mut ret = BFprogram {
            source_name: PathBuf::from(source_name.as_ref()),
            src: Vec::new(),
        };

        // Technically we should split on b'\n', b'\r\n', or '\r'.
        // b'\r\n' will leave a b'\r' at the end of the line, this will be consumed without issue.
        // b'\r' was only used as a line terminator by Macs, pre OS X. We'll assume that this won't
        // be an issue...
        for (line_number, line) in data.as_ref().split(|c| *c == b'\n').enumerate() {
            for (char_number, c) in line.iter().enumerate() {
                if let Some(inst) = Instruction::from_byte(*c) {
                    ret.src.push(InputInstruction {
                        inst,
                        line_number: line_number + 1,
                        char_number: char_number + 1,
                        source_name: PathBuf::from(source_name.as_ref()),
                    });
                }
            }
        }

        ret
    }

    /// `as_slice` allows us to access the underlying bytecode instructions.
    #[must_use]
    pub fn as_slice(&self) -> &[InputInstruction] {
        self.src.as_slice()
    }
}

impl AsRef<[InputInstruction]> for BFprogram {
    fn as_ref(&self) -> &[InputInstruction] {
        self.src.as_ref()
    }
}
