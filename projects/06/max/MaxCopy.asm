// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/06/max/Max.asm

// Computes R2 = max(R0, R1)  (R0,R1,R2 refer to RAM[0],RAM[1],RAM[2])
   @256
   @R0
   D=M              // D = first number
   @R1
   D=D-M            // D = first number - second number
   @OUTPUT_FIRST
   D;JGT            // if D>0 (first is greater) goto output_first
   @R1
   D=M              // D = second number
   @OUTPUT_D
   0;JMP            // goto output_d
(OUTPUT_FIRST)
   @R0             
   D=M              // D = first number
(OUTPUT_D)
   @R2
   M=D
   @END_EQ
   D;JNE
   @SP
   A=M-1
   @bat
   M=-1
(END_EQ)
   @SP
   AM=M-1
   D=M
   @THIS
   A=M+1
   A=A+1
   M=D
   @name
   0;JMP
   @sys.halt
   D=A
   @R14
   M=D
   @RET_ADDRESS_CALL326
   D=A
   @95
   0;JMP
   (RET_ADDRESS_CALL326)
   @SP
   AM=M-1
   @bat
   D=M
   @name
   @R5
   M=D
   (sys.halt)    // M[2] = D (greatest number)
(INFINITE_LOOP)
   @INFINITE_LOOP
   0;JMP            // infinite loop
