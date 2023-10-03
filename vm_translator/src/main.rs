use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::path::Path;

/// LABEL_COUNTER is used to keep track of the label count.
/// AtomicUsize ensures that the counter can be safely accessed and modified across
/// multiple threads if running program is multithreaded.
static LABEL_COUNTER: AtomicUsize = AtomicUsize::new(0);

// next_label_id is a placeholder for a function that generates unique label IDs.
// Label IDs are used in assembly languages to create unique labels for jumps and branching.
fn next_label_id() -> usize {
    LABEL_COUNTER.fetch_add(1, Ordering::Relaxed)
}


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
    let mut output_file =
        File::create(output_filename.to_string()).expect("Unable to create output file");

    // Get the filename without extension.
    let path = Path::new(&args[1]);
    let static_identifier = if let Some(filename) = path.file_stem() {
        filename.to_string_lossy()
    } else{
        eprintln!("No filename in path: {}", &args[1]);
        return;
    };

    // Process each line of the VM code.
    for line in reader.lines() {
        let line = line.expect("Error reading line");
        let parts: Vec<&str> = line.trim().split_whitespace().collect();

        if parts.is_empty() || line.starts_with("//") {
            // Skip empty lines or ignore comments.
            continue;
        }

        let command = parts[0];
        let arg1 = parts.get(1).cloned();
        let arg2 = parts.get(2).cloned();

        write!(&mut output_file, "// {}\n", line).expect("Error writing to output");

        match command {
            "add" | "sub" => {
                /*
                @SP: This line sets the A-register to the address pointed to by the Stack Pointer (SP). It's essentially telling the computer to access the value at the top of the stack.

                AM=M-1: This is a combination of two operations. A=M-1 sets the A-register to the address immediately below the current top of the stack. M=M-1 then decrements the value at the top of the stack. This is a common pattern for accessing and modifying the top value while leaving the Stack Pointer in the correct position for further operations.

                D=M: This line copies the value from the memory location pointed to by the A-register (which was set to the second-to-top value on the stack) into the D-register. This value is temporarily stored in the D-register for later use.

                A=A-1: This line decrements the A-register to point to the address immediately below the current top of the stack. Now, the A-register is pointing to the destination where we want to store the result of the addition.

                M=D+M: Finally, this line adds the value in the D-register (which holds the second value from the top of the stack) to the value in the memory location pointed to by the A-register (which is the top value of the stack). The result is stored back into the memory location pointed to by the A-register. In essence, this line replaces the two top values with their sum.
                */
                let op_symbol = if command == "add" { "+" } else { "-" };
                write!(
                    &mut output_file,
                    "\t@SP\n\
                    \tAM=M-1\n\
                    \tD=M\n\
                    \tA=A-1\n\
                    \tM=D{}M\n",
                    op_symbol
                )
                .expect("Error writing output");
            }
            "and" | "or" => {
                let logical_command = if command == "and" { "&" } else { "|" };

                write!(
                    &mut output_file,
                    "\t@SP\n\
                    \tAM=M-1\n\
                    \tD=M\n\
                    \tA=A-1\n\
                    \tM=M{}D\n",
                    logical_command
                )
                .expect("Error writing output");
            }
            "not" => {
                // Decrement the Stack Pointer (SP) and access the value at the top of the stack
                write!(
                    &mut output_file,
                    "@SP\n\
                    AM=M-1\n\
                    D=M\n"
                )
                .expect("Error writing to output");

                // Perform bitwise NOT operation on the value in the D register
                write!(
                    &mut output_file,
                    "D=!D\n"
                )
                .expect("Error writing to output");

                // Increment the Stack Pointer and store the result on the stack
                write!(
                    &mut output_file,
                    "@SP\n\
                    A=M\n\
                    M=D\n\
                    @SP\n\
                    M=M+1\n"
                )
                .expect("Error writing to output");
            }
            "neg" => {
                // Pop the value from the stack into the D register
                write!(
                    &mut output_file,
                    "@SP\n\
                    AM=M-1\n\
                    D=M\n"
                )
                .expect("Error writing to output");

                // Negate the value in the D register
                write!(
                    &mut output_file,
                    "@0\n\
                    D=A-D\n"
                )
                .expect("Error writing to output");

                // Push the negated value back onto the stack
                write!(
                    &mut output_file,
                    "@SP\n\
                    A=M\n\
                    M=D\n\
                    @SP\n\
                    M=M+1\n"
                )
                .expect("Error writing to output");
            }
            "eq" | "gt" | "lt" => {
                let jump_instruction = if command == "eq" {
                    "JEQ"
                } else if command == "gt" {
                    "JGT"
                } else {
                    "JLT"
                };

                let label_id = next_label_id();

                // Decrement SP and compare the top two stack values.
                write!(
                    &mut output_file,
                    "\t@SP\n\
                    \tAM=M-1\n\
                    \tD=M\n\
                    \tA=A-1\n\
                    \tD=M-D\n" // Make a subtraction, value will be 0 if they are equal
                )
                .expect("Error writing output");

                // Set the result to true (-1) if the values are equal; otherwise, set it to false (0).
                //We then perform the jump based on the comparison result, setting the
                // top of the stack accordingly (0 or -1) and jumping to the appropriate labels.
                write!(
                    &mut output_file,
                    "\t@TRUE.{}\n\
                    \tD;{}\n\
                    \t@SP\n\
                    \tA=M-1\n\
                    \tM=0\n\
                    \t@CONTINUE.{}\n\
                    \t0;JMP\n\
                    (TRUE.{})\n\
                    \t@SP\n\
                    \tA=M-1\n\
                    \tM=-1\n\
                    (CONTINUE.{})\n",
                    label_id, jump_instruction, label_id, label_id, label_id
                )
                .expect("Error writing output");
            }
            "push" => {
                // [push segment index]. Push the value of segment[index] onto the stack
                // where segment is argument, local, static, constant, this, that, pointer,
                // or temp and index is a positive integer.

                let segment = arg1.expect("Missing segment argument");
                let index = arg2.expect("Missing index argument");

                match segment {
                    "argument" => {
                        /*
                        Load the base address of the "argument" segment (which is stored in the ARG register) into the D register.

                        Add the index to the base address (loaded into the A register) to calculate the target address within the "argument" segment.
                        The value at the calculated target address is loaded into the D register.

                        Finally, the value from the D register is stored onto the stack, and the Stack Pointer (SP) is incremented to point to the next empty slot in the stack.
                         */
                        write!(
                            &mut output_file,
                            "\t@ARG\n\
                            \tD=M\n\
                            \t@{}\n\
                            \tA=D+A\n\
                            \tD=M\n\
                            \t@SP\n\
                            \tA=M\n\
                            \tM=D\n\
                            \t@SP\n\
                            \tM=M+1\n",
                            index
                        )
                        .expect("Error writing to output");
                    }
                    "local" => {
                        /*
                        Load the base address of the "local" segment (which is stored in the LCL register) into the D register.

                        Add the index to the base address to calculate the target address within the "local" segment.
                        The value at the calculated target address is loaded into the D register.

                        Finally, the value from the D register is stored onto the stack, and the Stack Pointer (SP) is incremented to point to the next empty slot in the stack.
                         */
                        write!(
                            &mut output_file,
                            "\t@LCL\n\
                            \tD=M\n\
                            \t@{}\n\
                            \tA=D+A\n\
                            \tD=M\n\
                            \t@SP\n\
                            \tA=M\n\
                            \tM=D\n\
                            \t@SP\n\
                            \tM=M+1\n",
                            index
                        )
                        .expect("Error writing to output");
                    }
                    "static" => {
                        /*
                        Each reference to "static index" will be translated to assembly symbol "FileName.index"

                        This label is used to access static variables in the assembly code.
                        It loads the value from the memory location pointed to by the label (which represents the static variable) into the D register.
                        The value from the D register is then stored onto the stack, and the Stack Pointer (SP) is incremented to point to the next empty slot in the stack.
                        */
                        let static_label = format!("{}.{}", static_identifier, index);
                        write!(
                            &mut output_file,
                            "\t@{}\n\
                            \tD=M\n\
                            \t@SP\n\
                            \tA=M\n\
                            \tM=D\n\
                            \t@SP\n\
                            \tM=M+1\n",
                            static_label
                        )
                        .expect("Error writing to output");
                    }
                    "constant" => {
                        /*
                        Uses the provided index value directly as the constant to be pushed onto the stack.
                        It loads the constant value into the D register.
                        The value from the D register is then stored onto the stack.
                        */
                        write!(
                            &mut output_file,
                            "\t@{}\n\
                            \tD=A\n\
                            \t@SP\n\
                            \tA=M\n\
                            \tM=D\n\
                            \t@SP\n\
                            \tM=M+1\n",
                            index
                        )
                        .expect("Error writing to output");
                    }
                    "this" => {
                        /*
                        Loads the base address of the "this" segment (which is stored in the THIS register) into the D register.
                        It then adds the index to the base address to calculate the target address within the "this" segment.
                        The value at the calculated target address is loaded into the D register.
                        Finally, the value from the D register is stored onto the stack
                        */
                        write!(
                            &mut output_file,
                            "\t@THIS\n\
                            \tD=M\n\
                            \t@{}\n\
                            \tA=D+A\n\
                            \tD=M\n\
                            \t@SP\n\
                            \tA=M\n\
                            \tM=D\n\
                            \t@SP\n\
                            \tM=M+1\n",
                            index
                        )
                        .expect("Error writing to output");
                    }
                    "that" => {
                        /*
                        Loads the base address of the "that" segment (which is stored in the THAT register) into the D register.
                        It then adds the index to the base address to calculate the target address within the "this" segment.
                        The value at the calculated target address is loaded into the D register.
                        Finally, the value from the D register is stored onto the stack.
                        */
                        write!(
                            &mut output_file,
                            "\t@THAT\n\
                            \tD=M\n\
                            \t@{}\n\
                            \tA=D+A\n\
                            \tD=M\n\
                            \t@SP\n\
                            \tA=M\n\
                            \tM=D\n\
                            \t@SP\n\
                            \tM=M+1\n",
                            index
                        )
                        .expect("Error writing to output");
                    }
                    "pointer" => {
                        /*
                        Access to pointer 0 should result in accessing the THIS pointer and any access
                        to pointer 1 should result in accessing the THAT pointer. The pointer segment
                        contains exactly two values and is mapped directly to RAM locations 3 and 4,
                        these RAM locations are also called THIS and THAT respectively.
                        */
                        match index {
                            "0" => {
                                write!(
                                    &mut output_file,
                                    "\t@THIS\n\
                                    \tD=M\n\
                                    \t@SP\n\
                                    \tA=M\n\
                                    \tM=D\n\
                                    \t@SP\n\
                                    \tM=M+1\n"
                                )
                                .expect("Error writing to output");
                            }
                            "1" => {
                                write!(
                                    &mut output_file,
                                    "\t@THAT\n\
                                    \tD=M\n\
                                    \t@SP\n\
                                    \tA=M\n\
                                    \tM=D\n\
                                    \t@SP\n\
                                    \tM=M+1\n",
                                )
                                .expect("Error writing to output");
                            }
                            _ => {
                                eprintln!("Unsupported pointer index: {}", index);
                            }
                        }
                    }
                    "temp" => {
                        /*
                        It is a fixed 8-word segment that is mapped directly to RAM locations 5 - 12, index
                        varies from 0 to 7.
                        Push from the temp segment (R5-R12)
                        */
                        if let Ok(parsed_index) = index.parse::<i32>() {
                            if parsed_index < 0 || parsed_index > 7 {
                                eprintln!("Supplied index should be between 0 - 7");
                                return;
                            }

                            let temp_base_address = 5;
                            let temp_current_address = temp_base_address + parsed_index;

                            write!(
                                &mut output_file,
                                "\t@{}\n\
                                \tD=M\n\
                                \t@SP\n\
                                \tA=M\n\
                                \tM=D\n\
                                \t@SP\n\
                                \tM=M+1\n",
                                temp_current_address
                            )
                            .expect("Error writing to output");
                        } else {
                            eprintln!("Supplied index can't be parsed to an integer");
                        }
                    }
                    _ => {
                        eprintln!("Unsupported push segment: {}", segment);
                    }
                }
            }
            "pop" => {
                // [pop segment index]. Pop the value of segment[index] from the stack
                // where segment is argument, local, static, constant, this, that, pointer,
                // or temp and index is a positive integer.
                let segment = arg1.expect("Missing segment argument");
                let index = arg2.expect("Missing index argument");

                match segment {
                    "argument" => {
                        /*
                        Calculate the target address within the "argument" segment in a single step by adding the index to the value stored in the ARG register and storing it in the D register as the target address.

                        Use @R13 as a temporary register (variable) to store the target address.

                        Next, we use AM=M-1 to decrement the Stack Pointer (SP) and access the value at the top of the stack, storing it in the D register.

                        Finally, we use the stored target address in @R13 to store the value from the D register into the target address within the "argument" segment.
                        */

                        // Calculate the target address within the "argument" segment.
                        write!(
                            &mut output_file,
                            "\t@ARG\n\
                            \tD=M\n\
                            \t@{}\n\
                            \tA=D+A\n\
                            \tD=A\n", // D = Target Address
                            index
                        )
                        .expect("Error writing to output");

                        // Pop the value from the stack into the target address.
                        write!(
                            &mut output_file,
                            "\t@R13\n\
                            \tM=D\n\
                            \t@SP\n\
                            \tAM=M-1\n\
                            \tD=M\n\
                            \t@R13\n\
                            \tA=M\n\
                            \tM=D\n"
                        )
                        .expect("Error writing to output");
                    }
                    "local" => {
                        // Logic is similar to "argument" segment above with a
                        // difference of the base memory address LOCAL.
                        // Calculate the target address within the "argument" segment.
                        write!(
                            &mut output_file,
                            "\t@LCL\n\
                            \tD=M\n\
                            \t@{}\n\
                            \tA=D+A\n\
                            \tD=A\n", // D = Target Address
                            index
                        )
                        .expect("Error writing to output");

                        // Pop the value from the stack into the target address.
                        write!(
                            &mut output_file,
                            "\t@R13\n\
                            \tM=D\n\
                            \t@SP\n\
                            \tAM=M-1\n\
                            \tD=M\n\
                            \t@R13\n\
                            \tA=M\n\
                            \tM=D\n"
                        )
                        .expect("Error writing to output");
                    }
                    "static" => {
                        let static_label = format!("{}.{}", static_identifier, index);
                        // Pop the value from the stack into the D register.
                        // Decrement the Stack Pointer (SP) and access the value at
                        // the top of the stack. The value is then stored in the D register.
                        write!(
                            &mut output_file,
                            "\t@SP\n\
                            \tAM=M-1\n\
                            \tD=M\n"
                        )
                        .expect("Error writing to output");

                        // Store the popped value to the static address location.
                        write!(
                            &mut output_file,
                            "\t@{}\n\
                            \tM=D\n",
                            static_label
                        )
                        .expect("Error writing to output");
                    }
                    "constant" => {
                        /*
                        The constant segment in VM is read-only, meaning you can only push values onto the stack using it. It doesn't support the pop operation because it doesn't represent a writable memory location. Therefore, there is no need to implement the constant segment.
                        */
                        eprintln!("Unexpected pop operation: {}", segment);
                        return;
                    }
                    "this" => {
                        /*
                        Calculate the target address within the "this" segment in a single step by adding the index to the value stored in the THIS register and storing it in the D register as the target address.

                        Use @R13 as a temporary register (variable) to store the target address.

                        Next, we use AM=M-1 to decrement the Stack Pointer (SP) and access the value at the top of the stack, storing it in the D register.

                        Finally, we use the stored target address in @R13 to store the value from the D register into the target address within the "this" segment.
                        */

                        // Calculate the target address within the "this" segment
                        write!(
                            &mut output_file,
                            "\t@THIS\n\
                            \tD=M\n\
                            \t@{}\n\
                            \tA=D+A\n\
                            \tD=A\n", // D = Target Address
                            index
                        )
                        .expect("Error writing to output");

                        // Pop the value from the stack into the target address
                        write!(
                            &mut output_file,
                            "\t@R13\n\
                            \tM=D\n\
                            \t@SP\n\
                            \tAM=M-1\n\
                            \tD=M\n\
                            \t@R13\n\
                            \tA=M\n\
                            \tM=D\n"
                        )
                        .expect("Error writing to output");
                    }
                    "that" => {
                        // Logic is similar to this segment above.
                        write!(
                            &mut output_file,
                            "\t@THAT\n\
                            \tD=M\n\
                            \t@{}\n\
                            \tA=D+A\n\
                            \tD=A\n", // D = Target Address
                            index
                        )
                        .expect("Error writing to output");

                        // Pop the value from the stack into the target address
                        write!(
                            &mut output_file,
                            "\t@R13\n\
                            \tM=D\n\
                            \t@SP\n\
                            \tAM=M-1\n\
                            \tD=M\n\
                            \t@R13\n\
                            \tA=M\n\
                            \tM=D\n"
                        )
                        .expect("Error writing to output");
                    }
                    "pointer" => {
                        // Pop the value from the stack into the D register.
                        // Decrement the Stack Pointer (SP) and access the value at
                        // the top of the stack. The value is then stored in the D register.
                        write!(
                            &mut output_file,
                            "\t@SP\n\
                            \tAM=M-1\n\
                            \tD=M\n"
                        )
                        .expect("Error writing to output");

                        // Store the popped value in the D-register into the THIS or THAT pointer.
                        match index {
                            "0" => {
                                write!(
                                    &mut output_file,
                                    "\t@THIS\n\
                                    \tM=D\n"
                                )
                                .expect("Error writing to output");
                            }
                            "1" => {
                                write!(
                                    &mut output_file,
                                    "\t@THAT\n\
                                    \tM=D\n",
                                )
                                .expect("Error writing to output");
                            }
                            _ => {
                                eprintln!("Unsupported pointer index: {}", index);
                            }
                        }
                    }
                    "temp" => {
                        // Pop the value from the stack into the D register.
                        // Decrement the Stack Pointer (SP) and access the value at
                        // the top of the stack. The value is then stored in the D register.
                        write!(
                            &mut output_file,
                            "\t@SP\n\
                            \tAM=M-1\n\
                            \tD=M\n"
                        )
                        .expect("Error writing to output");

                        if let Ok(parsed_index) = index.parse::<i32>() {
                            if parsed_index < 0 || parsed_index > 7 {
                                eprintln!("Supplied index should be between 0 - 7");
                                return;
                            }

                            let temp_base_address = 5;
                            let temp_current_address = temp_base_address + parsed_index;

                            // Store the popped value in the D-register into the temp_current_address.
                            write!(
                                &mut output_file,
                                "\t@{}\n\
                                \tM=D\n",
                                temp_current_address
                            )
                            .expect("Error writing to output");
                        } else {
                            eprintln!("Supplied index can't be parsed to an integer");
                        }
                    }
                    _ => {
                        eprintln!("Unsupported pop segment: {}", segment);
                    }
                }
            }
            _ => {
                eprintln!("Unsupported VM command: {}", command);
            }
        }
    }

    // It is recommended to end each program with an infinite loop.
    write!(
        &mut output_file,
        "(INFINITE_LOOP)\n\
        \t@INFINITE_LOOP\n\
        \t0;JMP\n"
    )
    .expect("Error writing to output");

    println!("Finished VM Code translation to hack assembly code!");
}
