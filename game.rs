//! Ball game data structures, text input parser, transition function, win condition, display, etc.

// This is like a C++ using statement. it brings the following struct into scope.
use std::num::NonZeroU8;
use std::io::BufRead;
use std::fmt;
use std::collections::HashMap;
use std::borrow::Borrow;
use std::hash::Hash;
use crate::astar::{Score,State};
use std::rc::Rc;

/// A game state, consisting of a number of Tubes.
#[derive(Clone, Hash, Eq, PartialEq)] // Automatically generate code implementing `Clone`, a common trait (interface) for a type to implement.
pub struct Game {
    /// Three to thirteen tubes, according to assignment one's description.
    pub tubes: Vec<Tube>, // a dynamically sized vector of Tubes
}

/// A tube, containing up to four balls.
#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Tube {
    /// NOTE: Balls fall towards the 0th index of this array by gravity, although this interface doesn't enforce this property.
    pub balls: [Option<Ball>; 4], // an array of four Option<Ball>s. Each Option
}

/// A ball, identified by a single ASCII character.
/// There should be exactly three other balls with the same color in the same game as this ball.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Ball {
    /// Balls cannot have a color of '\0'.
    pub color: NonZeroU8, // a u8 is an unsigned char, and this NonZeroU8 is a u8 that can't be 0.
}

/// A game move, devoid of context.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Action {
    from: u8,
    to: u8,
}

impl Game {
    /// Parse input text into a Game object.
    /// See A1-input.txt as an example input
    ///
    /// # Panics
    ///
    /// Panics if BufRead::read_line fails on `input`, and on malformed input.
    // arg input is an object that implements the BufRead trait, which provides .read_line()
    // `-> Game` means we return a Game object.
    pub fn from_input(input: &mut dyn BufRead) -> Game {
        let mut tubes = Vec::with_capacity(13);
        let mut tubes_remaining = 0; // # of full tubes left to parse

        let mut full_count_parsed = false;
        let mut empty_count_parsed = false;
        loop{ // an infinite loop. has a break statement below, in EOF.
            let mut line = String::new();
            // read from input into `line`, up to newline (LF) or EOF
            let result = input.read_line(&mut line);

            // IO errors or non-text input will cause the program to exit here
            let bytes_read = result.expect("Game::from_input: Failed to read line from input stream. Potentially invalid UTF-8?");

            // EOF
            if bytes_read == 0 {
                if !full_count_parsed || !empty_count_parsed { panic!("EOF reached before end of header!"); }
                if tubes_remaining > 0 { panic!("EOF reached, but {} more tubes of ball colors are expected!", tubes_remaining); }
                // Happy exit from loop
                break;
            }

            // remove whitespace, because we might as well
            line = line.trim().to_string();

            // Skip empty lines
            if line.is_empty() { continue; }

            // Skip comments
            if line.starts_with("//") { continue; }

            if !full_count_parsed {
                // Line 1: # of full tubes
                full_count_parsed = true;
                tubes_remaining = line.parse().expect("Couldn't parse full tube count");
            } else if !empty_count_parsed {
                // Line 2: # of empty tubes
                empty_count_parsed = true;
                let empty_tubes = line.parse().expect("Couldn't parse empty tube count");
                for _ in 0..empty_tubes {
                    tubes.push(Tube::empty());
                }
            } else if tubes_remaining > 0 {
                tubes_remaining -= 1;
                // Remaining lines: Colors of balls
                if !line.is_ascii() { panic!("Unexpected non-ASCII character in ball color line. Line: {:?}", line); }
                if line.bytes().count() != 4 { panic!("Expected exactly 4 ascii characters for ball colors, got line: {:?}", line); }
                let mut colors = line.bytes();
                let parse_ball = |colors: &mut std::str::Bytes| { // A closure (lambda function)
                    let color = colors.next().unwrap();
                    if !color.is_ascii_graphic() { panic!("Unprintable character used as ball color. Hex {:#?}", color); }
                    Some(Ball{ color: NonZeroU8::new(color).unwrap() })
                };
                tubes.push(Tube{
                    balls: [
                        parse_ball(&mut colors),
                        parse_ball(&mut colors),
                        parse_ball(&mut colors),
                        parse_ball(&mut colors)
                    ]
                });
            } else {
                // Treat a non-empty line after the ball colors as a fatal error.
                panic!("Unexpected line at end of input: {:?}", line);
            }
        }
        let game = Game { // Construct a new Game object,
            tubes // with tubes variable as tubes member
        };
        if !game.is_valid() {
            panic!("Invalid game board. Do you have four balls of each color, 2-10 colors, and 1-3 empty tubes?");
        }
        game // returns game object, as there's no semicolon
    }

    pub fn try_action(&self, action: Action) -> Option<Self> {
        let from = action.from as usize;
        let to = action.to as usize;
        let len = self.tubes.len();
        // bounds check
        if from >= len || to >= len { return None; }
        // NOP check
        if from == to { return None; }
        // ensure there's a spot to go and a ball to take
        if !self.tubes[to].balls[3].is_none() || !self.tubes[from].balls[0].is_some() { return None; }

        let to_idx = self.tubes[to].last();
        let from_idx = self.tubes[from].last()-1;

        // expensive: clone state. (hence why we do all error handling before this)
        let mut new_state = self.clone();
        // move ball
        new_state.tubes[to].balls[to_idx] = new_state.tubes[from].balls[from_idx];
        new_state.tubes[from].balls[from_idx] = None;

        debug_assert!(!self.is_valid() || new_state.is_valid(), "try_action moved to an invalid state from valid state");

        Some(new_state)
    }

    /// Check if the game state follows the rules outlined in the assignment description.
    /// This operation is a tad costly, as it allocates a hashmap. It can be written without one, but I wrote it as simply as possible.
    pub fn is_valid(&self) -> bool {
        // TODO: If I were to return enum values like TooManyBallsOfColor or NotEnoughEmpties, the user error could be more specific.

        // A1 specifies valid games state with 2-10 full tubes and 1-3 empty, for a total of 3-13 tubes.
        if self.tubes.len() < 3  { return false; }
        if self.tubes.len() > 13 { return false; }

        // ensure there are no floating balls
        if !self.tubes.iter().all(Tube::is_valid) { return false; }

        let mut count = HashMap::new();
        for tube in &self.tubes {
            for ball in &tube.balls {
                *count.entry(ball).or_insert(0u8) += 1;
            }
        }

        for (ball, &count) in &count {
            if let Some(_ball) = ball {
                // The count of some ball
                if count != 4 { return false; }
            } else {
                // the "air" space count
                if count < 4 { return false; }
                if count > 12 { return false; }
            }
        }

        // All conditions met, ball is valid!
        true
    }
}

impl State for Game {
    type Edge = Action;
    type Iter = GameSuccessors;
    fn iter_successors(self) -> GameSuccessors {
        GameSuccessors {
            state: self,
            action: Action{
                from: 0,
                to: 0
            }
        }
    }
    fn try_edge(&self, edge: &Action) -> Option<Self> {
        self.try_action(*edge)
    }
    /// If a game `is_solved` and `is_valid`, then it is solved. Some invalid board states are considered solved by this function,
    /// but checking for validity is a tad costly.
    ///
    /// All successors to valid board states, as provided by iter_successors or try_action, will be valid board states.
    fn is_solved(&self) -> bool {
        // Apply the Tube::is_solved method to all our tubes, and return true iff all are solved.
        return self.borrow().tubes.iter().all(Tube::is_solved);
    }
}

impl State for Rc<Game> {
    type Edge = Action;
    type Iter = RcGameSuccessors;
    fn iter_successors(self) -> RcGameSuccessors {
        RcGameSuccessors {
            state: self,
            action: Action{
                from: 0,
                to: 0
            }
        }
    }
    fn try_edge(&self, edge: &Action) -> Option<Self> {
        self.try_action(*edge).map(Rc::new)
    }
    /// If a game `is_solved` and `is_valid`, then it is solved. Some invalid board states are considered solved by this function,
    /// but checking for validity is a tad costly.
    ///
    /// All successors to valid board states, as provided by iter_successors or try_action, will be valid board states.
    fn is_solved(&self) -> bool {
        // Apply the Tube::is_solved method to all our tubes, and return true iff all are solved.
        return self.tubes.iter().all(Tube::is_solved);
    }
}

/// An iterator over the successive states to a ball game state.
pub struct GameSuccessors {
    state: Game,
    action: Action
}

impl Iterator for GameSuccessors {
    type Item = (Game, Score, Action);
    fn next(&mut self) -> Option<Self::Item> {
        let len = self.state.tubes.len() as u8;

        // outer loop iterates self.action.from over 0..self.state.tubes.len()
        // inner loop iterates self.action.to over 0..self.state.tubes.len()
        while self.action.from < len {
            while self.action.to < len {
                // If we've stumbled into a valid move
                if let Some(new_state) = self.state.try_action(self.action) {
                    // cost of all moves in ball game is 1.
                    let cost = 1;
                    // copy action before modifying self.action
                    let action = self.action;
                    // increment so we don't keep yielding the same result
                    self.action.to += 1;

                    return Some((new_state, cost, action))
                }
                self.action.to += 1;
            }

            self.action.to = 0;
            self.action.from += 1;
        }
        None
    }
}

/// An iterator over the successive states to a ball game state.
pub struct RcGameSuccessors {
    state: Rc<Game>,
    action: Action
}

impl Iterator for RcGameSuccessors {
    type Item = (Rc<Game>, Score, Action);
    fn next(&mut self) -> Option<Self::Item> {
        let len = self.state.tubes.len() as u8;

        // outer loop iterates self.action.from over 0..self.state.tubes.len()
        // inner loop iterates self.action.to over 0..self.state.tubes.len()
        while self.action.from < len {
            while self.action.to < len {
                // If we've stumbled into a valid move
                if let Some(new_state) = self.state.try_action(self.action) {
                    // cost of all moves in ball game is 1.
                    let cost = 1;
                    // copy action before modifying self.action
                    let action = self.action;
                    // increment so we don't keep yielding the same result
                    self.action.to += 1;

                    return Some((Rc::new(new_state), cost, action))
                }
                self.action.to += 1;
            }

            self.action.to = 0;
            self.action.from += 1;
        }
        None
    }
}

impl Tube {
    pub fn empty() -> Tube {
        Tube{ balls: [None; 4] }
    }
    /// Returns true if there are 4 identical balls, or no balls in this tube.
    fn is_solved(&self) -> bool {
        // Construct an iterator of all pairs of balls, in the form (balls[N], balls[N+1]).
        let mut balls = self.balls.iter().zip(self.balls.iter().skip(1));
        // return true if all pairs of subsequent balls are equal
        balls.all(|(ball1, ball2)| ball1==ball2)
    }
    /// Returns true if all balls are following gravity (left-aligned)
    fn is_valid(&self) -> bool {
        // Construct an iterator of all pairs of balls, in the form (balls[N], balls[N+1]).
        let mut balls = self.balls.iter().zip(self.balls.iter().skip(1));
        // return true if no ball is preceded by air
        balls.all(|(ball1, ball2)| ball1.is_some() || ball2.is_none())
    }

    fn last(&self) -> usize {
        self.balls.iter().map(|b| if b.is_some() {1} else {0}).sum()
    }
}

// Implement Game, Tube, and Ball formatting strings
impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f,
            "Game with {} tubes, in {} state",
            self.tubes.len(),
            // If/else is an expression, not a statement.
            if !self.is_valid() { "an invalid" }
            else if !self.is_solved() { "a valid" }
            else { "a solved" }
        )?; // ? is the error short-circuiting operator. It will return from this function if write fails
        for (idx, tube) in self.tubes.iter().enumerate() {
            writeln!(f, "[{:>2}] {}", idx, tube)?
        }
        Ok(())
    }
}


impl fmt::Display for Tube {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ball in &self.balls {
            if let Some(ball) = ball {
                write!(f, "{}", ball)?
            } else {
                write!(f, " ")?
            }
        }
        Ok(())
    }
}

impl fmt::Display for Ball {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", std::char::from_u32(self.color.get().into()).unwrap())
    }
}

