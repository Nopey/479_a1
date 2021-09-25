//! Heuristics for the ball game
#![allow(unused)]
use crate::game::*;
use crate::astar::*;

/// Knows nothing, gives a 0 score to all states
///
/// Devolves A* pathfinding into breadth first search
// 1: Solved for 10 long path in 352546 work steps. work queue len: 1232156
// 2: Too Slow!
pub fn ignoramus(_: &Game) -> Score {
    0
}

/// Penalizes nonconsecutive colors
///
/// Admissable: At least one of the two colors will need to be removed
// 1: Solved for 10 long path in 1098 work steps. work queue len: 2596
// 2: Solved for 10 long path in 8086 work steps. work queue len: 68282
// 3: Solved for 15 long path in 12556 work steps. work queue len: 180954
// 4: Too Slow!
pub fn consecutive_enjoyer(game: &Game) -> Score {
    game.tubes.iter().map(|tube| {
            tube.balls.iter().zip(tube.balls.iter().skip(1))
            .map(|(ball1, ball2)| if ball1==ball2 {0} else {1})
            .sum::<Score>()
        }).sum()
}

/// This cost heuristic is the number of balls that are not
/// connected to the bottom of the bucket by a run of one color.
///
/// Admissable: All solutions will move the non-matching balls out of the bucket
// 1: Solved for 10 long path in 296 work steps. work queue len: 1090
// 2: Solved for 10 long path in 53082 work steps. work queue len: 855667
// 3: Too Slow!
pub fn count_clutter(game: &Game) -> Score {
    let mut cost = 0;
    for tube in &game.tubes {
        let mut balls = tube.balls.iter();
        let mut floor = *balls.next().unwrap();
        for &ball in balls {
            if ball.is_none() { break; }
            if ball != floor {
                cost += 1;
                floor = None; // all balls ontop of a mismatched color are gonna require moving
            }
        }
    }
    cost
}

/*
/// A relaxed version of the ball sorting game, in which.. I'll think of something
/// 
#[derive(Debug)]
struct RelaxedGame {
    game: Game,
}

impl State for RelaxedGame {
    type Edge = Action;
    type Iter = 
    fn is_solved(&self) -> bool {
        // use the unmodified game's rules for 
        game.is_solved()
    }
}
*/

/// Solves a relaxed version of the game in which there are more buckets, using the consecutive_enjoyer heuristic
///
/// Admissable: A* with an admissable cost heuristic will give a path no longer than the real solution.
// 1: Solved for 10 long path in 63 work steps. work queue len: 223
// 2: Solved for 10 long path in 33 work steps. work queue len: 503
pub fn relaxed_bucket_solve(game: &Game) -> Score {
    let mut relaxed_game = game.clone();
    relaxed_game.tubes.push(Tube::empty());
    relaxed_game.tubes.push(Tube::empty());
    relaxed_game.tubes.push(Tube::empty());
    let path = solve(relaxed_game, consecutive_enjoyer);
    path.map(|x| x.0.len() as Score).unwrap_or(0)
}

