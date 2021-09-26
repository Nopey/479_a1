//! Heuristics for the ball game
#![allow(unused)] // not every heuristic is used, and that's O-K
use crate::game::*;
use crate::astar::*;
use std::collections::HashSet;

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
// 1: Solved for 10 long path in 161 work steps. work queue len: 332
// 2: Solved for 10 long path in 3722 work steps. work queue len: 49073
// 3: Solved for 15 long path in 76961 work steps. work queue len: 1980371
// 4: Too Slow!
pub fn consecutive_enjoyer(game: &Game) -> Score {
    game.tubes.iter().map(|tube| {
            tube.balls.iter().zip(tube.balls.iter().skip(1))
            .map(|(ball1, ball2)| if ball2.is_none() || ball1==ball2 {0} else {1})
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

/// Similar to consecutive_enjoyer, but also dislikes repeated root balls of the same color
///
/// Admissable: all of those root balls of the same color need to be moved into one bucket eventually
// 1: Solved for 10 long path in 148 work steps. work queue len: 323
// 2: Solved for 10 long path in 224 work steps. work queue len: 2697
// 3: Solved for 15 long path in 282 work steps. work queue len: 6857
// 4: 
pub fn diggly(game: &Game) -> Score {
    let mut seen = HashSet::new();
    game.tubes.iter().map(|tube| {
            tube.balls.iter().zip(tube.balls.iter().skip(1))
            .map(|(ball1, ball2)| if ball2.is_none() || ball1==ball2 {0} else {1})
            .sum::<Score>()
            + tube.balls[0].map(|ball|
                if seen.get(&ball).is_some(){ 1 } else { seen.insert(ball); 0 }
            ).unwrap_or(0)
        }).sum()
}

/// Same heuristic as diggly, but is implemented without a hashset for greater performance.
///
/// # Panics
///
/// Panics if balls' colors are not compressed. (See: main.rs compress_game)
pub fn compressed_diggly(game: &Game) -> Score {
    // if there are N tubes, there are at most N-1 colors
    let mut seen = vec![false; game.tubes.len()];
    game.tubes.iter().map(|tube| {
            tube.balls.iter().zip(tube.balls.iter().skip(1))
            .map(|(ball1, ball2)| if ball2.is_none() || ball1==ball2 {0} else {1})
            .sum::<Score>()
            + tube.balls[0].map(|ball|
                if seen[ball.color.get() as usize]{ 1 } else { seen[ball.color.get() as usize] = true; 0 }
            ).unwrap_or(0)
        }).sum()
}


/// Similar to count_clutter, but with diggly's advantage over consecutive_enjoyer
///
/// Admissable: Each ball either needs to be moved (at least once), or can stay where it is.
/// For balls that are on the bottom of the tube, only one ball of each color can stay on the bottom of the tubes.
/// For all other balls, they can only stay if there is a continous run of one color to the bottom of the tube.
// 1: Solved for 10 long path in 66 work steps. work queue len: 198
// 2: Solved for 10 long path in 76 work steps. work queue len: 962
// 3: Solved for 15 long path in 135 work steps. work queue len: 3227
// 4: Solved for 25 long path in 57035 work steps. work queue len: 2093664
pub fn dig_clutter(game: &Game) -> Score {
    let mut seen = HashSet::new();
    game.tubes.iter().map(|tube| {
            // penalize balls not having a continous streak of one color connecting to the bottom
            tube.balls.iter().zip(tube.balls.iter().skip(1))
            .fold((0, false), |(cost, broken), (ball1, ball2)| {
                if ball2.is_some() && ( broken || ball1!=ball2 ) {
                    (cost + 1, true) // garbage balls we'll need to move
                } else {
                    (cost, false) // haven't broken continous streak, or we're in the air
                }
            }).0
            // penalize multiple tubes' bottom ball being the same color
            + tube.balls[0].map(|ball|
                if seen.get(&ball).is_some(){ 1 } else { seen.insert(ball); 0 }
            ).unwrap_or(0)
        }).sum()
}

/// Compressed version of dig_clutter
///
/// # Panics
///
/// Panics if balls' colors are not compressed. (See: main.rs compress_game)
pub fn compressed_dig_clutter(game: &Game) -> Score {
    // NOTE: this `seen` vector's 0 position goes unused. could be avoided, but reduces readability of below code.
    let mut seen = vec![false; game.tubes.len()];
    game.tubes.iter().map(|tube| {
            // penalize balls not having a continous streak of one color connecting to the bottom
            tube.balls.iter().zip(tube.balls.iter().skip(1))
            .fold((0, false), |(cost, broken), (ball1, ball2)| {
                if ball2.is_some() && ( broken || ball1!=ball2 ) {
                    (cost + 1, true) // garbage balls we'll need to move
                } else {
                    (cost, false) // haven't broken continous streak, or we're in the air
                }
            }).0
            // penalize multiple tubes' bottom ball being the same color
            + tube.balls[0].map(|ball|
                if seen[ball.color.get() as usize]{ 1 } else { seen[ball.color.get() as usize] = true; 0 }
            ).unwrap_or(0)
        }).sum()
}


/// Solves a relaxed version of the game in which there are more buckets, using the diggly heuristic
///
/// Admissable: A* with an admissable cost heuristic will give a path no longer than the real solution.
// 1: Solved for 10 long path in 148 work steps. work queue len: 323
// 2: Solved for 10 long path in 224 work steps. work queue len: 2697
// 3: Solved for 15 long path in 282 work steps. work queue len: 6857
// 4: too slow
// (Appears to be roughly equivalent to diggly)
pub fn relaxed_bucket_solve(game: &Game) -> Score {
    let mut relaxed_game = game.clone();
    relaxed_game.tubes.push(Tube::empty());
    relaxed_game.tubes.push(Tube::empty());
    relaxed_game.tubes.push(Tube::empty());
    let path = solve(relaxed_game, diggly);
    path.map(|x| x.0.len() as Score).unwrap_or(0)
}

