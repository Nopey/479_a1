# ball game and agent
  Assignment 1
  VIU CSCI 479
 Magnus L Larsen
  655 784 536


## Compilation
To compile, run `make`.
To make a debug build, run `make balls_dbg`
`make clean` is also supported.

Code documentation can be generated, run `make doc` to generate the documentation from the doc comments.
Once generated, the documentation can be found in the `doc` folder, start at `doc/balls/index.html`.

## Running
To run, simply enter `./balls A1-input1.txt`, replacing the argument with the desired input file.


## Output
After loading the input file, the program will display initial state.
Then, it will attempt to solve the problem.
If it successfully solves the problem, it will print out
1. some statistics about how difficult the problem was to solve, and the length of the solution (See: astar.rs's SolveStats),
2. the series of actions the agent has found to solve the ball game, and
3. the actions and the board state after the action has been taken.

Theoretically all valid game states have a solution and will be solved;
however, the agent may use up all available resources and crash with one of the following two messages:
*. memory allocation of ?????? bytes failedAborted
*. Killed
The first occurs when the agent runs out of memory, and the second when the agent uses too much CPU and gets killed (by some system process?)

If the agent does not run out of resources while solving an impossible problem, it will exhaust both all possible moves and all novel states.
Once it exhausts its options, the astar algorithm will give up, and the program's 'main' function will panic (exit) with the following message:
```
thread 'main' panicked at 'Couldn't solve ball game', src/libcore/option.rs:1188:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace.
```

The astar algorithm will terminate, as there are a finite number of unique states that can be visited, and no state is visited twice.


## Levels
The game ships with five levels, from 0 to 4 inclusive.

Levels 1 through 4 are the course's example inputs, found at http://csci.viu.ca/~liuh/479/assignments/A1-input.txt

Level 0 is similar to Level 1, but is already in a win-state.


### Level Format
Leading and trailing whitespace is ignored.
After whitespace trimming, blank lines and lines starting with two forward slashes are ignored.

The first line is the number of non-empty tubes, a positive integer in the range of 3 to 10 inclusive.
The second line is the the number of empty tubes, a positive integer in the range of 1 to 3 inclusive.

The remaining lines are the ball colors, which can be any ASCII graphic, hex value ranging from 0x21 to 0x7E inclusive.
There are four balls per line, and each line is a single tube's balls.
The leftmost character is the bottom ball's color, and the rightmost character is the top ball's color.

Only one level can be specified per file, so the A1-input.txt provided on the website will fail, as it contains 4 levels.


## Game State
Game state is defined in game.rs, as structs Game, Tube, and Ball.

The Game owns a vector of Tubes.
Each Tube owns an array of Four Optional Balls (each one can be present, or not).
Each Ball is a non-zero unsigned 8-bit integer.
(When loading from a file, ball colors have to be in the ASCII graphic range of 0x21..0x7e inclusive)

Why are colors non-zero? Because zero is used for air.

There are invalid states that can be formed using these types, and so to check the validity of a game state, the is_valid function is defined on the Game object.
It checks that balls come in groups of four, that the number of tubes is reasonable, and other rules taken from the assignment description (See game.rs for more details).
This `Game::is_valid` function is called after loading user input, to ensure all user inputs are reasonable.

To improve the efficiency of some heuristics, the Game::compress function will replace all of the nice graphical ascii character colors with values starting at 1.
This allows vectors to be used instead of hashmaps, with the color value being used as an index.


## Winning
A state is considered final when the `Game::is_solved` function defined in game.rs returns `true`.
Game::is_solved checks if each tube is filled with one color or is completely empty.
If any tube is neither of those things, the game state is not solved and not final.


## Successor Function
The successor function is implemented in game.rs.

To find the successors to a state,
first the State iter_successors implemented on Game is called,
which returns a GameSuccessors object.

The GameSuccessors object implements the Iterator trait,
and the `next` method is responsible for generating the successors.
It does this by going through every pair of tubes starting at tube 0 and tube 0,
and checking whether it is a valid action (move).

Game::try_action is responsible for determining if an action is valid,
and generating the resulting state.

First, try_action checks that the action is in-range, and doesn't reference tubes that don't exist.
Secondly, try_action checks that the action isn't a no-op, where the source and destination match.
Thirdly, try_action ensures the destination has a spot for a ball and the source has a ball to take.
Finally, try_action generates the resulting state by actually moving the ball.


## Heuristics
Heuristics are defined in the h10s.rs file.
I define several heuristics that go unused, but were part of my development process.
The most effective heuristic is `dig_clutter`, and its somewhat faster twin `compressed_dig_clutter`.

The compressed version of the heuristic is implemented with a vector instead of a hashset,
but requires the (reasonably cheap) Game::compress preprocessing step.
(See the Game Data section for more information on Game::compress, or read about the function in game.rs)


## Selection of most promising state
The most promising state is found with a priority queue in `astar.rs`.
Rust's standard library includes a priority queue, BinaryHeap, and it pops the largest value.

We're using `Node`'s in the BinaryHeap to ensure that the largest node is the node with the smallest `astar_cost`.
astar_cost is the path cost plus the heuristic cost.
Node has a custom `Ord` implementation to do the Ordering, which can be found at the bottom of astar.rs


## Errors
Malformed input will generate an error message such as the following:
```
thread 'main' panicked at 'Invalid game board: NotEnoughBallsOfColor', game.rs:138:13
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace.
```

## Bugs
There are no bugs currently known.
