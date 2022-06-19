# Nand-Tetris

In the quest to go deeper in the stack(low-level) and do a bottom-up approach, this project contains my implementation of building a 16-bit modern computer systems using Logic Gates / Chips which is what the modern computer architecture is made of.

In the process of studying this amazing book [The Element of Computing Systems](https://www.amazon.co.uk/Elements-Computing-Systems-second-Principles/dp/0262539802/ref=asc_df_0262539802/?tag=googshopuk-21&linkCode=df0&hvadid=430805552339&hvpos=&hvnetw=g&hvrand=1093056745030148753&hvpone=&hvptwo=&hvqmt=&hvdev=c&hvdvcmdl=&hvlocint=&hvlocphy=9046245&hvtargid=pla-918789568003&psc=1&th=1&psc=1&tag=&ref=&adgrpid=101598702938&hvpone=&hvptwo=&hvadid=430805552339&hvpos=&hvnetw=g&hvrand=1093056745030148753&hvqmt=&hvdev=c&hvdvcmdl=&hvlocint=&hvlocphy=9046245&hvtargid=pla-918789568003)

## Implementation

I will be using a specification, Hardware Description Language (HDL) to build my hardware computers using different logic gates combination.

## First Project

Create the following Chips:

- Not (Inverter): This gate outputs the opposite value of it's input, something like `!true == false` in a programming language. `done`

- And: returns 1 when both its inputs are 1, and 0 if they are not. `done`

- Or: returns 1 when at least one of its input is 1, and 0 otherwise `done`

- Xor (Exclusive Or): returns 1 when exactly one of its inputs is 1, and 0 otherwise `done`

- Multiplexer: it is a three-input gate, it uses `sel` bit to select and output either of the 2 supplied inputs. `done`

- Demultiplexer: It is the opposite of Multiplexer, it takes a single inout value and routes it to one of two possible outputs according to a selection bit that selects the destination output. `done`

- 16-bit Not (Not16) `done`

- 16-bit And (And16) `done`

- 16-bit Or (Or16) `done`

- 16-bit Multiplexer (Mux16) `done`

- Multi-way Or (Or8Way)

- Multi-way Multiplexer (Mux4Way16, Mux8Way16)

- Mult-way Demultiplexer (DMux4Way, DMux8Way)

Implementation: [Project 1](https://github.com/Lumexralph/nand-tetris/tree/main/projects/01)


