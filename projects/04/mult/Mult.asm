// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Mult.asm

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)
//
// This program only needs to handle arguments that satisfy
// R0 >= 0, R1 >= 0, and R0*R1 < 32768.

    // set sum variable to 0
    @sum
    M=0

    // initialize i (loop counter) to R1.
    @R1
    D=M
    @i
    M=D

// start a loop that uses addition to achieve the multiplication.
// if R0 = x and R1 = y, initialize the loop variable i = y,
// keep decrementing till i == 0
(LOOP)
    // loop conditonal check, if i == 0
    @i
    // reduce i by 1.
    D=M-1
    M=D
    @STOP
    D; JLT

    // loop body. (sum += R0)
    @R0
    D=M
    @sum
    M=D+M
    // goto LOOP
    @LOOP
    0; JMP

(STOP)
    // set the value in variable sum to R2.
    @sum
    D=M
    @R2
    M=D
(END)
    @END
    0; JMP
