use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

// TODO: change all the expect() error handling to a reliable pattern.
fn main() {
    // Check if a VM file is provided as a command-line argument.
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <InputFile.vm>", args[0]);
        std::process::exit(1);
    }

    // Open the input VM file for reading.
    let input_file = File::open(&args[1]).expect("Unable to open input file");
    let reader = BufReader::new(input_file);

    // Open the output Hack assembly file for writing.
    let output_filename = format!("{}.asm", &args[1].split('.').next().unwrap());
    let mut output_file = File::create(output_filename).expect("Unable to create output file");

    // Initialize a counter for generating unique labels.
    let mut label_counter = 0;

    // Process each line of the VM code.
    for line in reader.lines() {
        let line = line.expect("Error reading line");
        let parts: Vec<&str> = line.trim().split_whitespace().collect();

        if parts.is_empty() {
            continue; // Skip empty lines.
        }

        let command = parts[0];
        let arg1 = parts.get(1).cloned();
        let arg2 = parts.get(2).cloned();

        match command {
            "add" => {
                /*
                @SP: This line sets the A-register to the address pointed to by the Stack Pointer (SP). It's essentially telling the computer to access the value at the top of the stack.

                AM=M-1: This is a combination of two operations. A=M-1 sets the A-register to the address immediately below the current top of the stack. M=M-1 then decrements the value at the top of the stack. This is a common pattern for accessing and modifying the top value while leaving the Stack Pointer in the correct position for further operations.

                D=M: This line copies the value from the memory location pointed to by the A-register (which was set to the second-to-top value on the stack) into the D-register. This value is temporarily stored in the D-register for later use.

                A=A-1: This line decrements the A-register to point to the address immediately below the current top of the stack. Now, the A-register is pointing to the destination where we want to store the result of the addition.

                M=D+M: Finally, this line adds the value in the D-register (which holds the second value from the top of the stack) to the value in the memory location pointed to by the A-register (which is the top value of the stack). The result is stored back into the memory location pointed to by the A-register. In essence, this line replaces the two top values with their sum.
                */
                write!(&mut output_file, "// {}\n", line).expect("Error writing to output");
                write!(
                    &mut output_file,
                    "@SP\n\
                    AM=M-1\n\
                    D=M\n\
                    A=A-1\n\
                    M=D+M\n"
                ).expect("Error writing output");
            }
            "sub" => {
                // Implement subtraction (similar to "add").
            }
            "push" => {
                // Implement push operation.
            }
            "pop" => {
                // Implement pop operation.
            }
            _ => {
                eprintln!("Unsupported VM command: {}", command);
            }
        }
    }

    println!("Hello, world!");
}
