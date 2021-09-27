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

No moving of source code is needed, the directory structure is flat
(with the exception of the doc folder for documentation, which is automatically created).

### Running

To run, simply enter `./balls A1-input1.txt`, replacing the argument with the desired input file.


## Levels
The game ships with five levels, from 0 to 4 inclusive.

Levels 1 through 4 are the course's example inputs, found at http://csci.viu.ca/~liuh/479/assignments/A1-input.txt

Level 0 is similar to Level 1, but is already in a win-state.

### Level Format
Leading and trailing whitespace is ignored.
After whitespace trimming, blank lines and lines starting with two forward slashes are ignored.

The first line is the number of non-empty tubes, a positive integer in the range of 3 to 10 inclusive.
The second line is the the number of empty tubes, a positive integer in the range of 1 to 3 inclusive.

The remaining lines are the ball colors, which can be any ASCII graphic,
hex value ranging from 0x21 to 0x7E inclusive.

There are four balls per line, and each line is a single tube's balls.
The leftmost character is the bottom ball's color,
and the rightmost character is the top ball's color.

Only one level can be specified per file, so the A1-input.txt provided on the website will fail,
as it contains 4 levels.


## Output
After loading the input file, the program will display initial state.
Then, it will attempt to solve the problem.
If it successfully solves the problem, it will print out
1. some statistics about how difficult the problem was to solve
    and the length of the solution (See: astar.rs's SolveStats),
2. the series of actions the agent has found to solve the ball game, and
3. the actions and the board state after the action has been taken.

Theoretically all valid game states have a solution and will be solved;
however, the agent may use up all available resources and crash with one of the following two messages:
*. memory allocation of ?????? bytes failedAborted
*. Killed

The first occurs when the agent runs out of memory,
and the second when the agent uses too much CPU and gets killed (by some system process?)

If the agent does not run out of resources while solving an impossible problem,
it will exhaust both all possible moves and all novel states.
Once it exhausts its options, the astar algorithm will give up,
and the program's 'main' function will panic (exit) with the following message:
```
thread 'main' panicked at 'Couldn't solve ball game', src/libcore/option.rs:1188:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace.
```

The astar algorithm will terminate when run on the ball game,
as there are a finite number of unique states that can be visited, and no state is visited twice.


## Game State
Game state is defined in game.rs, as structs Game, Tube, and Ball.

The Game owns a vector of Tubes.
Each Tube owns an array of Four Optional Balls (each one can be present, or not).
Each Ball is a non-zero unsigned 8-bit integer.
(When loading from a file, ball colors have to be in the ASCII graphic range of 0x21..0x7e inclusive)

Why are colors non-zero? Because zero is used for air.

There are invalid states that can be formed using these types, and so to check the validity of a game state,
the is_valid function is defined on the Game object.
It checks that balls come in groups of four, that the number of tubes is reasonable,
and other rules taken from the assignment description (See game.rs for more details).
This `Game::is_valid` function is called after loading user input, to ensure all user inputs are reasonable.

To improve the efficiency of some heuristics,
the Game::compress function will replace all of the nice graphical ascii character colors with values starting at 1.
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

### ignoramus
The first heuristic I implemented, and the simplest.
It knows nothing about ball game, and assigns all states an equal cost of 0.
It performs the worst, as it reduces A* to breadth first search.
This is trivially admissable, as no path has less than 0 cost.

### consecutive_enjoyer
The second heuristic I implemented.
It iterates through each tube's balls bottum-up,
and assigns 1 cost to any ball that immediately follows a ball of a different color.
This is admissable, as any ball following a ball of mismatched color must be moved eventually.

### count_clutter
This heuristic counts the number of balls that do not form a streak of one color at the bottom of the bucket.
These must be moved, as any ball not matching the color of the bottom of the bucket must be moved,
and all balls above it must be moved. It is akin to consecutive_enjoyer,
but also penalizes balls that match the bottom of the bucket but are trapping mismatched balls below.
This is admissable, as a mismatched ball needs all balls ontop of it to be removed in order 

It dominates consecutive_enjoyer, because it is both admissable,
and only adds cost (whereever a ball is the same color as the root but not connected by a streak).

### diggly
This heuristic is like consecutive_enjoyer, but additionally adds a cost of 1 for every duplicate root ball color.
This is admissable, because consecutive_enjoyer doesn't ever penalize the root, and all duplicate roots will
eventually have to move.

It dominates consecutive_enjoyer because it is both admissable and adds cost at the duplicate root balls.

diggly is the first to have a compressed variant,
which is logically equivalent, but uses a vector rather than a hashset,
and requires the game to be compressed.

### dig_clutter
This heuristic is to count_clutter as diggly is to consective_enjoyer;
it adds 1 cost per duplicated root,
and is admissable because count_clutter assigns no cost to the root balls.

It dominates both diggly and count_clutter:
* For count_clutter, it adds cost at the duplicated roots (as diggly did to consecutive_enjoyer)
* For diggly, it adds cost wherever a ball is hte same color as the root, but is not connected by a streak.
Thus, dig_clutter indirectly dominates all of the hand-crafted heuristics.

Like diggly, it has a compressed version compressed_dig_clutter.
I use compressed_dig_clutter as the heuristic in balls,
the others are just there to show my development and thought process.

### relaxed_bucket_solve
This heuristic is fairly slow, and was actually the third heuristic I implemented.
I could have implemented a modified successor function to make a more interesting relaxed version of the ball game,
but didn't have any inspiration as to what rule could be relaxed to make a reasonable heuristic.

I favor compressed_dig_clutter over relaxed_bucket_solve due to the speed of the handcrafted heuristic.


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
There are no currently known bugs.

Previously, the agent would use up all of the resources and crash with one of the two out of resource errors,
see the Output section; this may occur on inputs I haven't tested,
but I have greatly reduced memory usage and improved my heuristics since I last experienced these errors.

## Comments
I have written this assignment in Rust.
Feel free to reach out if you have any questions regarding my assignment or the Rust language itself.
