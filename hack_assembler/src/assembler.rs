use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::collections::HashMap;

/// Assembler reads the hack assembly program using
/// the provided path to the file.
pub struct Assembler {
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
        // pre-create R0 -> R15.
        for value in 0..=15 {
            self.symbol_table.insert(format!("R{value}"), value);
        }

        println!("{:?}", self.symbol_table);
    }

    pub fn read_file(&self) -> std::io::Result<()> {
        let f = File::open(&self.path)?;
        let reader = BufReader::new(f);
        let mut line_no = 0;

        for line in reader.lines() {
            match line {
                Ok(content) => { println!("{line_no}: {}", content); }
                Err(error) => { println!("error: {}", error); }
            }
            line_no = line_no + 1;
        }

        Ok(())
    }
}
