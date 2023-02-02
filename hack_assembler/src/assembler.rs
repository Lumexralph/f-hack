use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read, Seek};
use std::path::PathBuf;

use lazy_static::lazy_static;
use regex::Regex;

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
    // For SCREEN, KBD (Keyboard)
    Devices((&'static str, u16)),
}

lazy_static! {
    // Regex match: dest=comp;jump OR dest=comp
    static ref RE: Regex = Regex::new("^.*?=.*?(;.)?$").unwrap();
    static ref NUM_RE: Regex = Regex::new("^[0-9]+$").unwrap();
}

/// Code module provides services for translating symbolic
/// Hack mnemonics into their binary codes.
#[derive(Default)]
struct Code<'b> {
    dest: HashMap<&'b str, String>,
    jump: HashMap<&'b str, String>,
    comp: HashMap<&'b str, &'b str>,
}

impl<'b> Code<'b> {
    fn new() -> Self {
        let mut dest = HashMap::new();
        let dest_instruction_set = ["null", "M", "D", "DM", "A", "AM", "AD", "ADM"];
        for (index, instruction) in dest_instruction_set.iter().enumerate() {
            dest.insert(*instruction, format!("{index:03b}"));
        }

        let mut jump = HashMap::new();
        let jump_instruction_set = ["null", "JGT", "JEQ", "JGE", "JLT", "JNE", "JLE", "JMP"];
        for (index, instruction) in jump_instruction_set.iter().enumerate() {
            jump.insert(*instruction, format!("{index:03b}"));
        }

        let mut comp = HashMap::new();
        comp.insert("0", "101010");
        comp.insert("1", "111111");
        comp.insert("-1", "111010");
        comp.insert("D", "001100");
        comp.insert("A", "110000");
        comp.insert("M", "110000");
        comp.insert("!D", "001101");
        comp.insert("!A", "110001");
        comp.insert("!M", "110001");
        comp.insert("-D", "001111");
        comp.insert("-A", "110011");
        comp.insert("-M", "110011");
        comp.insert("D+1", "011111");
        comp.insert("A+1", "110111");
        comp.insert("M+1", "110111");
        comp.insert("D-1", "001110");
        comp.insert("A-1", "110010");
        comp.insert("M-1", "110010");
        comp.insert("D+A", "000010");
        comp.insert("D+M", "000010");
        comp.insert("D-A", "010011");
        comp.insert("D-M", "010011");
        comp.insert("A-D", "000111");
        comp.insert("M-D", "000111");
        comp.insert("D&A", "000000");
        comp.insert("D&M", "000000");
        comp.insert("D|A", "010101");
        comp.insert("D|M", "010101");

        Code { dest, jump, comp }
    }
}

/// Parser handles the reading and breaking of the hack asm
/// instructions into their underlying fields or types.
///
/// A_INSTRUCTION for @xxx, xxx is a decimal or symbol (variable or constants).
/// L_INSTRUCTION for (xxx), where xxx is a symbol.
/// C_INSTRUCTION for instructions of this format dest=comp;jump.
pub struct Parser<'a> {
    symbol_table: &'a mut HashMap<String, u16>,
    // Variable address starts from 16 and it is incremented
    // by 1 whenever another variable is encountered.
    variable_address: u16,
    instruction_line: u16,
    c_instruction_set: Code<'a>,
}

impl<'a> Parser<'a> {
    fn new(table: &'a mut HashMap<String, u16>) -> Self {
        Parser {
            symbol_table: table,
            variable_address: 16,
            instruction_line: 0,
            c_instruction_set: Code::new(),
        }
    }

    fn reset_instruction_line(&mut self) {
        self.instruction_line = 0
    }

    /// parse_labels() goes through the entire assembly program line by line,
    /// it keeps track of line number of code from 0 and is incremented by 1 whenever
    /// an A_INSTRUCTION or C_INSTRUCTION is found, but does not change when whitespace,
    /// comments or label declaration is encountered.
    /// It adds a new entry to the symbol table for label declaration (L_INSTRUCTION),
    /// associating the symbol with the current line number + 1 (this will be the ROM address
    /// of the next instruction in the program). No binary code is generated.
    fn parse_labels(&mut self, raw_content: &str) {
        let mut content = String::from(raw_content.replace(" ", ""));
        if content == "" || content.starts_with("//") {
            return;
        }

        // Remove in-line comments "//"
        content = match content.split_once("//") {
            Some((raw_content, _)) => raw_content.to_string(),
            None => content,
        };

        // Handle L_INSTRUCTION
        if content.starts_with("(") && content.ends_with(")") {
            let label = &content[1..content.len() - 1];
            println!("{} L_INSTRUCTION: {label}", self.instruction_line + 1);
            self.symbol_table
                .insert(label.to_string(), self.instruction_line + 1);
        } else {
            // Assumes the remaining instructions are C  and A INSTRUCTIONS.
            self.instruction_line = self.instruction_line + 1;
        }
    }

    /// parse_instructions reads the entire assembly code again, it handles the
    /// A and C INSTRUCTIONS and generates the binary code that will be sent
    /// to the computer processor.
    fn parse_instructions(&mut self, raw_content: &str) {
        let mut content = String::from(raw_content.replace(" ", ""));
        // Ignore whitespace, comment and labels (L_INSTRUCTIONS)
        if content == "" || content.starts_with("//") || content.starts_with("(") {
            return;
        }

        // Remove in-line comments "//"
        let refined_content = match content.split_once("//") {
            Some((raw_content, _)) => raw_content,
            None => content.as_str(),
        };
        content = refined_content.to_string();

        // Assumes only A and C INSTRUCTIONS are left after the
        // the ignored contents above i.e. comments, whitespace and labels.
        if content.starts_with("@") {
            self.decode_a_instructions(content);
            return;
        }

        // Possibly C-INSTRUCTION or invalid content.
        self.decode_c_instruction(content);

        self.instruction_line = self.instruction_line + 1;
    }

    fn decode_a_instructions(&mut self, content: String) {
        // Handle A-instructions
        let a_instruction = &content[1..];
        if NUM_RE.is_match(a_instruction) {
            println!(
                "{} A-INSTRUCTION (number): {a_instruction}",
                self.instruction_line
            );
        } else {
            match self.symbol_table.get(a_instruction) {
                Some(value) => {
                    // TODO: create the binary instruction of the value
                    println!(
                        "{}: variable already initialized {}:{}",
                        self.instruction_line, a_instruction, value
                    )
                }
                None => {
                    //TODO: You need to check if the new variable location is not SCREEN or KBD
                    // Initialize the new variable and increase the variable address.
                    self.symbol_table
                        .insert(a_instruction.to_string(), self.variable_address);
                    println!(
                        "{} A-INSTRUCTION (symbol: new variable): {a_instruction}: {}",
                        self.instruction_line, self.variable_address
                    );
                    // TODO: create the binary instruction of the value (variable_address)

                    self.variable_address += 1;
                }
            }
        }
    }

    fn decode_c_instruction(&self, instruction: String) {
        let mut content = instruction;
        if content.contains("=") && RE.is_match(content.as_str()) {
            // Cut the dest part of content.
            match content.split_once("=") {
                Some((dest, remaining_substr)) => {
                    let dest_code = match self.c_instruction_set.dest.get(dest) {
                        Some(code) => code,
                        None => {
                            println!("error: invalid dest {dest} instruction provided!");
                            return;
                        }
                    };
                    println!("{} dest: {dest} : {dest_code}", self.instruction_line);
                    content = remaining_substr.to_string();
                }
                None => {}
            };
        }
        if content.contains(";") {
            match content.split_once(";") {
                Some((comp, jump)) => {
                    let comp_code = match self.c_instruction_set.comp.get(comp) {
                        Some(code) => code,
                        None => {
                            println!("error: invalid comp {comp} instruction provided!");
                            return;
                        }
                    };

                    let jump_code = match self.c_instruction_set.jump.get(jump) {
                        Some(code) => code,
                        None => {
                            println!("error: invalid jump {jump} instruction provided!");
                            return;
                        }
                    };

                    println!(
                        "{} comp;jump => {comp};{jump} : {comp_code};{jump_code}",
                        self.instruction_line
                    );
                }
                None => {}
            };
        } else {
            // Assumes content will be comp if none of the dest and jump conditions match.
            let comp_code = match self.c_instruction_set.comp.get(content.as_str()) {
                Some(code) => *code,
                None => {
                    println!("error: invalid comp {content} instruction provided!");
                    return;
                }
            };
            println!("comp: {content}:{comp_code}");
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
    // HashMap<symbol, address>
    symbol_table: HashMap<String, u16>,
    /// The path to the .asm file to read.
    pub(crate) path: PathBuf,
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
        // Pre-create R0 -> R15 registers.
        for address in 0..=15 {
            self.symbol_table.insert(format!("R{address}"), address);
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

        // Add special labels to the symbol table.
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
    }

    pub fn read_file(&mut self) -> io::Result<()> {
        let f = File::open(&self.path)?;
        let mut reader = BufReader::new(f);
        let mut parser = Parser::new(&mut self.symbol_table);

        // First pass.
        for line in reader.by_ref().lines() {
            match line {
                Ok(content) => {
                    parser.parse_labels(content.as_str());
                }
                Err(error) => println!("error reading file content: {error}"),
            }
        }

        if let Err(result) = reader.rewind() {
            return Err(result);
        }

        // Second pass.
        parser.reset_instruction_line();
        for line in reader.by_ref().lines() {
            match line {
                Ok(content) => {
                    parser.parse_instructions(content.as_str());
                }
                Err(error) => println!("error reading file content {error}"),
            }
        }

        println!("{:?}", self.symbol_table);
        Ok(())
    }
}
