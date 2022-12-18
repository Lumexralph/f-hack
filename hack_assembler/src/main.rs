use clap::Parser;

mod assembler;

/// Args reads the command line arguments.
#[derive(Parser, Debug)]
struct Args {
    /// The hack assembly language input file.
    path: std::path::PathBuf,
}

fn main() {
    let args = Args::parse();

    if args.path.exists() {
       // TODO: check file extension is .asm

        let asmbler = assembler::Assembler{ path: args.path };
        asmbler.read_file();
    }

}
