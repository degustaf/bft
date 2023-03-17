//! Types for a Brainf*ck interpreter.

#![warn(missing_docs)]
use std::cmp::Eq;
use std::convert::AsRef;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::read;
use std::io;
use std::path::{Path, PathBuf};

/// Raw bytecodes for the brainf*ck VM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputInstruction {
    inst: Instruction,
    line_number: usize,
    char_number: usize,
}

impl InputInstruction {
    /// A string representation of the instruction's location in the file.
    #[must_use]
    pub fn location(&self) -> String {
        format!("{}:{}", self.line_number, self.char_number)
    }

    /// Extract the underlying instruction.
    #[must_use]
    pub fn instruction(&self) -> &Instruction {
        &self.inst
    }
}

/// Possibile errors during the bracket matching algorithm.
#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum BracketMatchError {
    /// An opening bracket never found a matching closing bracket.
    ExtraOpeningBracket(PathBuf, usize, usize),

    /// A closing bracket was found when all opening brackets were matched.
    ExtraClosingBracket(PathBuf, usize, usize),
}

impl Display for BracketMatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            Self::ExtraClosingBracket(source_name, line_number, char_number) => {
                write!(
                    f,
                    "Unexpected closing bracket ']' at [{}:{}:{}]",
                    source_name.display(),
                    line_number,
                    char_number
                )
            }
            Self::ExtraOpeningBracket(source_name, line_number, char_number) => {
                write!(
                    f,
                    "Unmatched bracket '[' at [{}:{}:{}]",
                    source_name.display(),
                    line_number,
                    char_number
                )
            }
        }
    }
}

impl Error for BracketMatchError {}

/// A container to hold an entire Brainf*ck program.
#[derive(Debug)]
pub struct BFprogram {
    source_name: PathBuf,
    src: Vec<InputInstruction>,
    brackets: Vec<(usize, usize)>,
}

impl BFprogram {
    /// Load data from a source file, and parse it into bytecode.
    ///
    /// # Errors
    /// This function will return an error if opening the file fails, or if there is an error
    /// reading the bytes within the file.
    pub fn from_file<P: AsRef<Path>>(file_name: P) -> io::Result<Self> {
        let data = read(&file_name)?;
        Ok(Self::new(file_name, &data))
    }

    /// Parse Extended ASCII text into Brainf*ck bytecode. The Path `source_name` is used to store
    /// the name of the source of the text.
    /// ```
    /// use bft_types::BFprogram;
    /// let code = Vec::from(" <  > [\n]");
    /// let program = BFprogram::new("doc.test", &code);
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
    pub fn new<P: AsRef<Path>>(source_name: P, data: &[u8]) -> BFprogram {
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
            brackets: Vec::new(),
        }
    }

    /// `instructions` allows us to access the underlying bytecode instructions.
    #[must_use]
    pub fn instructions(&self) -> &[InputInstruction] {
        self.src.as_slice()
    }

    /// get the name of the source file for the program.
    #[must_use]
    pub fn source(&self) -> &PathBuf {
        &self.source_name
    }

    /// Validate the program by ensuring that the brackets match.
    ///
    /// # Errors
    /// This function will return an error if it fails to validate brackets with an error
    /// describing where the error occurred and what is the porblem.
    ///
    /// ```
    /// use bft_types::BFprogram;
    /// let code = Vec::from(" <  > [\n]");
    /// let mut program = BFprogram::new("doc.test", &code);
    ///
    /// assert!(program.validate_brackets().is_ok());
    /// ```
    pub fn validate_brackets(&mut self) -> Result<(), BracketMatchError> {
        let mut stack: Vec<usize> = Vec::new();
        let mut brackets: Vec<(usize, usize)> = Vec::new();

        for (idx, inst) in self.src.iter().enumerate() {
            match *inst.instruction() {
                Instruction::BeginLoop => {
                    stack.push(idx);
                }
                Instruction::EndLoop => match stack.pop() {
                    Some(matched_bracket) => {
                        brackets.push((matched_bracket, idx));
                    }
                    _ => {
                        return Err(BracketMatchError::ExtraClosingBracket(
                            self.source_name.clone(),
                            inst.line_number,
                            inst.char_number,
                        ));
                    }
                },
                _ => {}
            }
        }

        if let Some(idx) = stack.pop() {
            let inst = self.src[idx];
            Err(BracketMatchError::ExtraOpeningBracket(
                self.source_name.clone(),
                inst.line_number,
                inst.char_number,
            ))
        } else {
            self.brackets = brackets;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_display() {
        assert_eq!(
            format!("{}", Instruction::MoveLeft),
            "Move left one location"
        );
        assert_eq!(
            format!("{}", Instruction::MoveRight),
            "Move right one location"
        );
        assert_eq!(
            format!("{}", Instruction::Increment),
            "Increment current location"
        );
        assert_eq!(
            format!("{}", Instruction::Decrement),
            "Decrement current location"
        );
        assert_eq!(
            format!("{}", Instruction::Input),
            "Accept one byte of input"
        );
        assert_eq!(
            format!("{}", Instruction::Output),
            "Output the current byte"
        );
        assert_eq!(format!("{}", Instruction::BeginLoop), "Start looping");
        assert_eq!(format!("{}", Instruction::EndLoop), "Finish looping");
    }

    #[test]
    fn byte_parsing() {
        assert_eq!(Instruction::from_byte(b'<'), Some(Instruction::MoveLeft));
        assert_eq!(Instruction::from_byte(b'>'), Some(Instruction::MoveRight));
        assert_eq!(Instruction::from_byte(b'+'), Some(Instruction::Increment));
        assert_eq!(Instruction::from_byte(b'-'), Some(Instruction::Decrement));
        assert_eq!(Instruction::from_byte(b','), Some(Instruction::Input));
        assert_eq!(Instruction::from_byte(b'.'), Some(Instruction::Output));
        assert_eq!(Instruction::from_byte(b'['), Some(Instruction::BeginLoop));
        assert_eq!(Instruction::from_byte(b']'), Some(Instruction::EndLoop));
    }

    #[test]
    fn bytes_that_dont_parse() {
        for i in (0..=255u8).filter(|n| !b"+,-.<>[]".contains(n)) {
            assert!(Instruction::from_byte(i).is_none());
        }
    }

    #[test]
    fn location() {
        let inst = InputInstruction {
            inst: Instruction::Increment,
            line_number: 100,
            char_number: 42,
        };
        assert_eq!(inst.location(), "100:42");
    }

    #[test]
    fn bracket_match_error_display() {
        assert_eq!(
            format!(
                "{}",
                BracketMatchError::ExtraOpeningBracket("mod.test".into(), 12, 45)
            ),
            "Unmatched bracket '[' at [mod.test:12:45]"
        );
        assert_eq!(
            format!(
                "{}",
                BracketMatchError::ExtraClosingBracket("mod.test".into(), 42, 78)
            ),
            "Unexpected closing bracket ']' at [mod.test:42:78]"
        );
    }

    #[test]
    fn proper_brackets() {
        let code = Vec::from("[[[]][][[[]]]]");
        let mut program = BFprogram::new("mod.test", &code);
        assert!(program.validate_brackets().is_ok());
    }

    #[test]
    fn missing_left_bracket() {
        let code = Vec::from("[[][][]]]");
        let mut program = BFprogram::new("mod.test", &code);
        assert_eq!(
            program.validate_brackets(),
            Err(BracketMatchError::ExtraClosingBracket(
                "mod.test".into(),
                1,
                9
            ))
        );
    }

    #[test]
    fn missing_right_bracket() {
        let code = Vec::from("[[][][]");
        let mut program = BFprogram::new("mod.test", &code);
        assert_eq!(
            program.validate_brackets(),
            Err(BracketMatchError::ExtraOpeningBracket(
                "mod.test".into(),
                1,
                1
            ))
        );
    }

    #[test]
    fn out_of_order_pairs() {
        let code = Vec::from("[[]]][");
        let mut program = BFprogram::new("mod.test", &code);
        assert_eq!(
            program.validate_brackets(),
            Err(BracketMatchError::ExtraClosingBracket(
                "mod.test".into(),
                1,
                5
            ))
        );
    }

    #[test]
    fn loading_from_file() {
        let file_name: PathBuf = PathBuf::from("../data/session1.txt");
        let program = BFprogram::from_file(file_name).expect("Program should load.");
        assert_eq!(*program.source(), PathBuf::from("../data/session1.txt"));
        let mut iter = program.instructions().iter();
        let inst = iter.next();
        assert_eq!(
            inst,
            Some(&InputInstruction {
                inst: Instruction::Increment,
                line_number: 8,
                char_number: 4
            })
        );
    }
}
