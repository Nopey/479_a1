//! Ball game data structures, text input parser, successor function, win condition, display, etc.

// This is like a C++ using statement. it brings the following struct into scope.
use std::num::NonZeroU8;
use std::io::BufRead;
use std::fmt;
use std::collections::HashMap;
use std::hash::Hash;
use crate::astar::{Score,State};

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

/// 
#[derive(Debug)]
pub enum ValidationError {
    /// 3 to 13 tubes must be present.
    NotEnoughTubes,
    /// 3 to 13 tubes must be present.
    TooManyTubes,
    /// No ball can have an empty spot below it
    SpaceBalls,
    /// Exactly 4 balls of each color group must be present
    NotEnoughBallsOfColor,
    /// Exactly 4 balls of each color group must be present
    TooManyBallsOfColor,
    /// 4, 8, or 12 empty spots must be present
    NotEnoughEmpties,
    /// 4, 8, or 12 empty spots must be present
    TooManyEmpties,
    // Game::from_input's parsing errors are treated as irrecoverable and fatal
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
        if let Err(e) = game.validate() {
            panic!("Invalid game board: {:?}", e);
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

        debug_assert!(self.validate().is_err() || new_state.validate().is_ok(), "try_action moved to an invalid state from valid state");

        Some(new_state)
    }

    /// Check if the game state follows the rules outlined in the assignment description.
    /// This operation is a tad costly, even using a hashmap; It can be written without one, but I wrote it as simply as possible.
    pub fn validate(&self) -> Result<(), ValidationError> {
        use ValidationError::*;
        // A1 specifies valid games state with 2-10 full tubes and 1-3 empty, for a total of 3-13 tubes.
        if self.tubes.len() < 3  { return Err(NotEnoughTubes); }
        if self.tubes.len() > 13 { return Err(TooManyTubes); }

        // ensure there are no floating balls
        // (balls preceded by an empty space)
        if !self.tubes.iter().all(Tube::is_valid) { return Err(SpaceBalls); }

        let mut count = HashMap::new();
        for tube in &self.tubes {
            for ball in &tube.balls {
                *count.entry(ball).or_insert(0u8) += 1;
            }
        }

        for (ball, &count) in &count {
            if let Some(_ball) = ball {
                // The count of some ball
                if count < 4 { return Err(NotEnoughBallsOfColor); }
                if count > 4 { return Err(TooManyBallsOfColor); }
            } else {
                // the "air" space count
                if count < 4 { return Err(NotEnoughEmpties); }
                if count > 12 { return Err(TooManyEmpties); }
            }
        }

        // All conditions met, game is valid!
        Ok(())
    }

    /// Replaces the printable colors with serialized ones, starting at 0x01.
    ///
    /// Afterwards, the game is not displayable, but several heuristics can use a more efficient vector implementation instead of hashset.
    pub fn compress(&mut self) {
        let mut idx = 0u8;
        // mapping from printable ASCII to new color id
        let mut hm = HashMap::new();
        // fill mapping
        for tube in &self.tubes {
            for ball in &tube.balls {
                if let Some(ball) = ball {
                    hm.entry(ball.color).or_insert_with(|| {
                        idx += 1;
                        NonZeroU8::new(idx).unwrap()
                    });
                }
            }
        }
        // replace all colors according to `hm`
        for tube in &mut self.tubes {
            for ball in &mut tube.balls {
                if let Some(ball) = ball {
                    ball.color = hm[&ball.color];
                }
            }
        }
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
/*
        // Noisier version, prints out game info above board
        writeln!(f,
            "Game with {} tubes, in {} state",
            self.tubes.len(),
            // If/else is an expression, not a statement.
            if self.validate().is_err() { "an invalid" }
            else if !self.is_solved() { "a valid" }
            else { "a solved" }
        )?; // ? is the error short-circuiting operator. It will return from this function if write fails
*/
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
        let c = self.color.get();
        if c < 0x10 {
            write!(f, "{:x}", c)
        } else {
            write!(f, "{}", std::char::from_u32(c.into()).unwrap())
        }
    }
}

