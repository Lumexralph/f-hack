use std::fs;

/// Assembler reads the hack assembly program using
/// the provided path to the file.
pub struct Assembler {
    /// The path to the .asm file to read.
    pub(crate) path: std::path::PathBuf,
}

impl Assembler {
    pub fn read_file(&self) {
        let content = fs::read_to_string(&self.path).expect("could not read file");
        let mut line_no = 0;

        for line in content.lines() {
            println!("{line_no} : {:}", line);
            line_no = line_no + 1;
        }
    }
}
