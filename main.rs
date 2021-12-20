//! ball game and agent
//!
//! Magnus Larsen 2021

mod game; // includes another source file, "game.rs". Namespaced to game:: 
mod astar;
mod h10s;

use std::fs::File;
use std::io::{BufRead, BufReader};

use astar::State;

/// Handles commandline interface and program lifecycle
fn main() {
    // Parse commandline arg
    let filename = std::env::args().skip(1).next().unwrap_or_else(||{
        // or print help
        eprintln!("balls: Expected one argument: input filename (or '-' for stdin)");
        std::process::exit(1)
    });

    // Read file, parse board, and display initial state
    let mut file_maybe = None;
    let mut stdin_maybe = None;
    let mut stdin_lock_maybe = None;
    let mut input: &mut dyn BufRead  = if filename == "-" {
        // NOTE: Because the cubs and pups only have Rust 1.41,
        // I can't use Option::insert, but get_or_insert can be used.
        stdin_lock_maybe.get_or_insert(
            stdin_maybe.get_or_insert(std::io::stdin()).lock()
        )
    } else {
        file_maybe.get_or_insert(
            BufReader::new(
                File::open(filename)
                    .expect("Couldn't open input file for reading")
            )
        )
    };
    let game = game::Game::from_input(&mut input);
    println!("Initial Board State:\n{}", &game);


    // Compress game to allow more efficient heuristics implementation
    let mut compressed_game = game.clone();
    compressed_game.compress();
    // run search
    // let (path, stats) = astar::solve(compressed_game, h10s::teenagent).expect("Couldn't solve ball game");
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

        // NOISY - Print out board after every move
        // println!("## {:?}:\n{}", action, state);
        // Still noisy, but slightly better:
        println!("{}", state);
    }
    if !state.is_solved() { panic!("Solution did not solve game!"); }
}
