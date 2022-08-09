// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Fill.asm

// Runs an infinite loop that listens to the keyboard input.
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel;
// the screen should remain fully black as long as the key is pressed. 
// When no key is pressed, the program clears the screen, i.e. writes
// "white" in every pixel;
// the screen should remain fully clear as long as no key is pressed.

(LOOP)
    @KBD
    D=M
    @input
    M=0 // set the data input to white
    @INNERLOOP
    D; JEQ // if keyboard input is 0

    @KBD
    D=M
    @input
    M=-1 // set the data input to black
    @INNERLOOP
    D; JNE // if keyboard is not 0

    @i
    M=0
    // inner loop.
    (INNERLOOP)
        @i
        D=M
        @SCREEN
        D=A+D // pick the next memory/pixel

        @input
        A=D // address of next pixel.
        // TODO: read from the keyboard input
        // it currently sets all the screen pixel to black.
        M=-1

        // increment i
        @i
        M=M+1
        D=M
        @INNERLOOP
        D; JGT

        //TODO: create a base index to break
        // out of the inner loop

        // break out of the inner loop.
        @END
        0; JMP

(END)
    // go to the beginning of the loop.
    @LOOP
    0; JMP
