mod game; // includes another source file, "game.rs". Namespaced to game:: 
mod astar;
mod h10s;

use std::fs::File;
use std::io::BufReader;
use std::num::NonZeroU8;
use std::collections::HashMap;
use std::rc::Rc;
use std::borrow::Borrow;

use game::Game;
use astar::State;

/// Entrypoint. Handles commandline arguments
fn main() {
    // Parse commandline arg
    let filename = std::env::args().skip(1).next().unwrap_or_else(||{
        // or print help
        eprintln!("Expected one argument: input filename");
        std::process::exit(1)
    });

    // Read file, parse board, and display initial state
    let mut ifile = File::open(filename).expect("Couldn't open input file for reading");
    let game = game::Game::from_input(&mut BufReader::new(&mut ifile));
    print!("{}", &game);


    // Compress game to allow more efficient heuristics implementation
    let mut compressed_game = game.clone();
    compress_game(&mut compressed_game);
    // run search
    let (path, stats) = astar::solve(Rc::new(compressed_game), |s| h10s::compressed_dig_clutter(s.borrow())).expect("Couldn't solve ball game");
    // let (path, stats) = astar::solve(Rc::new(compressed_game), |s| h10s::compressed_diggly(s.borrow())).expect("Couldn't solve ball game");


/*
    // Heuristics: (that don't require compress_game)
    // let heuristic = h10s::ignoramus;
    // let heuristic = h10s::consecutive_enjoyer;
    // let heuristic = h10s::count_clutter;
    // let heuristic = h10s::diggly;
    let heuristic = h10s::dig_clutter;
    // let heuristic = h10s::relaxed_bucket_solve;

    // Non reference counting solve (consumes more memory, is probably slower)
    // let (path, stats) = astar::solve(game.clone(), heuristic).expect("Couldn't solve ball game");

    let (path, stats) = astar::solve(Rc::new(game.clone()), |s| heuristic(s.borrow())).expect("Couldn't solve ball game");
*/

    println!("{}", stats);
    for edge in &path {
        println!("{:?}", edge);
    }
    println!();

    // Replay the path onto the starting position;
    // both to check that the path is a real solution,
    // and display the board state at each step.
    let mut state = game;
    for action in path {
        state = state.try_action(action).expect("Couldn't replay action from path");
        // println!("{}", heuristic(&state));

        // NOISY - Print out board after every move
        println!("## {:?}:\n{}", action, state);
    }
    if !state.is_solved() { panic!("Solution did not solve game!"); }
}

/// Replaces the printable colors with serialized ones, starting at 0x01.
///
/// The resulting game is not displayable, but someheuristics can use a more efficient vector implementation instead of hashset.
fn compress_game(game: &mut Game) {
    let mut idx = 0u8;
    let mut hm = HashMap::new();
    for tube in &game.tubes {
        for ball in &tube.balls {
            if let Some(ball) = ball {
                hm.entry(ball.color).or_insert_with(|| {
                    idx += 1;
                    NonZeroU8::new(idx).unwrap()
                });
            }
        }
    }
    for tube in &mut game.tubes {
        for ball in &mut tube.balls {
            if let Some(ball) = ball {
                ball.color = hm[&ball.color];
            }
        }
    }
}
