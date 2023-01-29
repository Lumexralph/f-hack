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
        match args.path.extension() {
            None => {}
            Some(ext) => {
                if ext != "asm" {
                    println!("invalid file format provided {:?}, expected .asm", ext);
                    return;
                }
            }
        }

        let mut asmbler = assembler::Assembler::new( args.path );
        asmbler.initialize();

        match asmbler.read_file() {
            Ok(_) => { println!("done!")}
            Err(err) => {println!("err: {err}")}
        }
    }

}
