# ball game
  Assignment 1
  VIU CSCI 479
 Magnus L Larsen
  655 784 536


## Compilation
To compile, run `make`.
To make a debug build, run `make balls_d`.
`make clean` is also supported.

Code documentation can be generated, run `make doc` to generate the documentation from the doc comments.
Once generated, the documentation can be found in the `doc` folder, start at `doc/balls/index.html`.

## Running
To run, simply enter `./balls A1-input1.txt`, replacing the argument with the desired input file.


## Levels (And how to make your own)
The game ships with five levels, from 0 to 4 inclusive.

Levels 1 through 4 are the course's example inputs, found at http://csci.viu.ca/~liuh/479/assignments/A1-input.txt

Level 0 is similar to Level 1, but is already in a win-state.

## Game State
Game state is defined in game.rs, as structs Game, Tube, and Ball.

The Game owns a vector of Tubes.
Each Tube owns an array of Four Optional Balls (each one can be present, or not).
Each Ball is a non-zero unsigned 8-bit integer.
(When loading from a file, ball colors have to be in the ASCII graphic range of 0x21..0x7e inclusive)

There are invalid states that can be formed using these types, and so to check the validity of a game state, the is_valid function is defined on the Game object.
It checks that balls come in groups of four, that the number of tubes is reasonable, and other rules taken from the assignment description (See game.rs for more details).
This `Game::is_valid` function is called after loading user input, to ensure all user inputs are reasonable.

## Winning
A state is considered final when the `Game::is_solved` function defined in game.rs returns `true`.
Game::is_solved checks if each tube is filled with one color or is completely empty.
If any tube is neither of those things, the game state is not solved and not final.

## Heuristics
Heuristics are defined in the h10s.rs file.
I define 

## Errors
Malformed input will trigger one of several messages:
* 
* 

## Bugs
There are no bugs currently known.
