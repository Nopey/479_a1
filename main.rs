//! ball game and agent
//!
//! Magnus Larsen 2021

mod game; // includes another source file, "game.rs". Namespaced to game:: 
mod astar;
mod h10s;

use std::fs::File;
use std::io::BufReader;

use astar::State;

/// Handles commandline intreface and program lifecycle
fn main() {
    // Parse commandline arg
    let filename = std::env::args().skip(1).next().unwrap_or_else(||{
        // or print help
        eprintln!("balls: Expected one argument: input filename");
        std::process::exit(1)
    });

    // Read file, parse board, and display initial state
    let mut ifile = File::open(filename).expect("Couldn't open input file for reading");
    let game = game::Game::from_input(&mut BufReader::new(&mut ifile));
    print!("{}", &game);


    // Compress game to allow more efficient heuristics implementation
    let mut compressed_game = game.clone();
    compressed_game.compress();
    // run search
    let (path, stats) = astar::solve(compressed_game, h10s::compressed_dig_clutter).expect("Couldn't solve ball game");
    // let (path, stats) = astar::solve(compressed_game, h10s::compressed_diggly).expect("Couldn't solve ball game");


/*
    // Heuristics: (that don't require compress_game)
    // let heuristic = h10s::ignoramus;
    // let heuristic = h10s::consecutive_enjoyer;
    // let heuristic = h10s::count_clutter;
    // let heuristic = h10s::diggly;
    let heuristic = h10s::dig_clutter;
    // let heuristic = h10s::relaxed_bucket_solve;

    // run search
    let (path, stats) = astar::solve(game.clone(), heuristic).expect("Couldn't solve ball game");
*/

    // Display stats and list path's edges
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

        // it was once useful to see the value of heuristics every step of the way
        // println!("{}", heuristic(&state));

        // NOISY - Print out board after every move
        println!("## {:?}:\n{}", action, state);
    }
    if !state.is_solved() { panic!("Solution did not solve game!"); }
}
