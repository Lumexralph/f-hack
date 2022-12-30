use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Index;
use std::path::PathBuf;

/// Pre-created labels in the symbol table.
const SP: (&str, u8) = ("SP", 0);
const LCL: (&str, u8) = ("LCL", 1);
const ARG: (&str, u8) = ("ARG", 2);
const THIS: (&str, u8) = ("THIS", 3);
const THAT: (&str, u8) = ("THAT", 4);
const SCREEN: (&str, u16) = ("SCREEN", 16384);
const KBD: (&str, u16) = ("KBD", 24576); // Keyboard

#[derive(Debug)]
enum PreCreatedLabel {
    Special((&'static str, u8)),
    /// For SCREEN, KBD (Keyboard)
    Devices((&'static str, u16)),
}

/// Parser handles the reading and breaking of the hack asm
/// instructions into their underlying fields or types.
///
/// A_INSTRUCTION for @xxx, xxx is a decimal or symbol (variable or constants).
/// L_INSTRUCTION for (xxx), where xxx is a symbol.
/// C_INSTRUCTION for instructions of this format dest=comp;jump.
pub struct Parser {}

impl Parser {
    fn new() -> Self {
        Parser {}
    }

    /// parse_labels() goes through the entire assembly program line by line,
    /// it keeps track of line number of code from 0 and is incremented by 1 whenever
    /// an A_INSTRUCTION or C_INSTRUCTION is found, but does not change when whitespace,
    /// comments or label declaration is encountered.
    /// It adds a new entry to the symbol table for label declaration (L_INSTRUCTION),
    /// associating the symbol with the current line number + 1 (this will be the ROM address
    /// of the next instruction in the program). No binary code is generated.
    fn parse_labels(&self, reader: BufReader<File>, symbol_table: &mut HashMap<String, u16>) {
        let mut line_no = 0;

        for line in reader.lines() {
            match line {
                Ok(mut content) => {
                    // ignore whitespaces and comments.
                    content = String::from(content.replace(" ", ""));
                    if content == "" || content.starts_with("//") {
                        continue;
                    }

                    // remove in-line comments "//"
                    content = match content.split_once("//") {
                        Some((raw_content, _)) => raw_content.to_string(),
                        None => content,
                    };

                    // handle L_INSTRUCTION
                    if content.starts_with("(") && content.ends_with(")") {
                        let label = content.index(1..content.len() - 1);
                        println!("{} L_INSTRUCTION: {label}", line_no + 1);
                        symbol_table.insert(label.to_string(), line_no + 1);
                    } else {
                        // Assumes the remaining instructions are C  and A INSTRUCTIONS.
                        line_no = line_no + 1;
                    }
                }
                Err(error) => {
                    println!("error reading line {line_no}: {}", error);
                }
            }
        }
    }

    /// parse_instructions reads the entire assembly code again, it handles the
    /// A and C INSTRUCTIONS and generates the binary code that will be sent
    /// to the computer processor.
    fn parse_instructions(&self, reader: BufReader<File>, symbol_table: &mut HashMap<String, u16>) {
        for line in reader.lines() {
            match line {
                Ok(mut content) => {
                    // ignore whitespaces and comments.
                    content = String::from(content.replace(" ", ""));
                    if content == "" || content.starts_with("//") {
                        continue;
                    }

                    // remove in-line comments "//"
                    let refined_content = match content.split_once("//") {
                        Some((raw_content, _)) => raw_content,
                        None => content.as_str(),
                    };
                    content = refined_content.to_string();

                    if content.starts_with("(") && content.ends_with(")") {
                        // handle A-instructions
                        let next_instruction_line = line_no + 1;
                        println!("{next_instruction_line} LABEL: {content}");
                        continue;
                    }
                    if content.starts_with("@") {
                        // handle A-instructions
                        println!("{line_no} A-INSTRUCTION: {content}");
                    }

                    // possibly C-INSTRUCTION or invalid content
                    if content.contains("=") && RE.is_match(content.as_str()) {
                        // cut the dest part of content
                        match content.split_once("=") {
                            Some((dest, remaining_substr)) => {
                                println!("{line_no} dest: {dest}");
                                content = remaining_substr.to_string();
                            }
                            None => {}
                        };
                    }
                    if content.contains(";") {
                        match content.split_once(";") {
                            Some((comp, jump)) => {
                                println!("{line_no} comp;jump => {comp};{jump}");
                                content = comp.to_string();
                            }
                            None => {}
                        };
                    } else if !content.starts_with("@") && !content.starts_with("(") {
                        // assumes content will be comp if none
                        // of the dest and jump conditions match.
                        println!("comp: {content}");
                    }
                }
                Err(error) => {
                    println!("error reading line {line_no}: {}", error);
                }
            }
        }
    }
}

/// Assembler reads the hack assembly program using
/// the provided path to the file.
/// It is a two-pass assembler that reads the code twice
/// from start to end (needed because of some symbols that
/// can be used before defined or initialized, they are pre-initialized
/// before the actual binary code is generated).
pub struct Assembler {
    symbol_table: HashMap<String, u16>,
    /// The path to the .asm file to read.
    pub(crate) path: PathBuf,
}

lazy_static! {
    // regex match: dest=comp;jump OR dest=comp
    static ref RE: Regex = Regex::new("^.*?=.*?(;.)?$").unwrap();
}

impl Assembler {
    /// Creates a new Assembler.
    pub fn new(file_path: PathBuf) -> Self {
        Assembler {
            symbol_table: HashMap::new(),
            path: file_path,
        }
    }

    /// initialize() creates a symbol table and initializes it with
    /// all the predefined symbols and their pre-allocated values.
    pub fn initialize(&mut self) {
        // pre-create R0 -> R15.
        for value in 0..=15 {
            self.symbol_table.insert(format!("R{value}"), value);
        }

        let special_labels = vec![
            PreCreatedLabel::Special(SP),
            PreCreatedLabel::Special(LCL),
            PreCreatedLabel::Special(ARG),
            PreCreatedLabel::Special(THIS),
            PreCreatedLabel::Special(THAT),
            PreCreatedLabel::Devices(SCREEN),
            PreCreatedLabel::Devices(KBD),
        ];

        // add special labels to the symbol table.
        for label in &special_labels {
            match label {
                PreCreatedLabel::Special(label) => self
                    .symbol_table
                    .insert(String::from(label.0), label.1 as u16),
                PreCreatedLabel::Devices(label) => {
                    self.symbol_table.insert(String::from(label.0), label.1)
                }
            };
        }
        println!("{:?}", self.symbol_table);
    }

    pub fn read_file(&mut self) -> std::io::Result<()> {
        let f = File::open(&self.path)?;
        let reader = BufReader::new(f);
        let parser = Parser::new();
        parser.parse_labels(reader, &mut self.symbol_table);

        println!("{:?}", self.symbol_table);
        Ok(())
    }
}
