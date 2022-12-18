use std::fs::File;
use std::io::{BufRead, BufReader};

/// Assembler reads the hack assembly program using
/// the provided path to the file.
pub struct Assembler {
    /// The path to the .asm file to read.
    pub(crate) path: std::path::PathBuf,
}

impl Assembler {
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
