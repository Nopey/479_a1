mod game; // links another source file, "game.rs". Namespaced to game:: 
mod astar;
mod h10s;

use std::fs::File;
use std::io::BufReader;
use std::num::NonZeroU8;
use std::collections::HashMap;
use game::Game;

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

    // let heuristic = h10s::ignoramus;
    let heuristic = h10s::consecutive_enjoyer;
    // let heuristic = h10s::count_clutter;
    // let heuristic = h10s::relaxed_bucket_solve;

/*
    // Compress game to make heuristics more efficient
    let mut compressed_game = game.clone();
    compress_game(&mut compressed_game);

    let path = astar::solve(compressed_game, heuristic).expect("Couldn't solve ball game");
*/
    let (path, stats) = astar::solve(game.clone(), heuristic).expect("Couldn't solve ball game");
    println!("{}", stats);
    // println!("{:?}", path);

    let mut state = game;
    for action in path {
        state = state.try_action(action).expect("Couldn't replay action from path");
        // print!("## {:?}:\n{}", action, state);
    }
}

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
