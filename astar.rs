//! Solve ball game with A* search
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;

/// An interface to expose a game's successor function to the search alg.
///
/// Implemented by struct Game in game.rs
// TODO: Remove Debug markers from astar code
pub trait State: Sized + Hash + Eq + Clone {
    /// The search algorithm will return the path as a vector of Edges.
    type Edge: fmt::Debug + Clone + Clone;
    /// An iterator over the neighboring states, their cost, and the 'Edge' to return if this is used as the solution Path.
    type Iter: Iterator<Item = (Self, Score, Self::Edge)>;
    /// Return an iterator over the neighboring states 
    fn iter_successors(self) -> Self::Iter;

    fn is_solved(&self) -> bool;
}

/// A typedef for the integer I'm using to keep track of score and cost
//TODO: Rename Score to `Cost`
pub type Score = i32;

/// An interface to a static heuristic function.
///
/// This interface is implemented by types in the h10s.rs file.
pub trait Heuristic<S: State> {
    fn score(state: &S) -> Score;
}

/// A simple wrapper around a State and a heuristic's score, that implements the Ord (Ordering) trait,
/// which allows the states to be sorted by their score in a binary heap.
struct Node<S: State> {
    /// Heuristic's score
    score: Score,
    /// Path cost to this path finding node
    cost: Score,
    /// this node's game state
    state: S,
    /// Previous edge and game state
    prev: Option<(S,S::Edge)>,
}

/// Statistics about how difficult the problem was to solve
pub struct SolveStats {
    pub path_len: usize,
    pub work_count: usize,
    pub work_queue_len: usize,
}

impl fmt::Display for SolveStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Solved for {} long path in {} work steps. work queue len: {}", self.path_len, self.work_count, self.work_queue_len)
    }
}

/// A generic implementation of A*, which takes an initial state and heuristic.
///
pub fn solve<S: State, H: Fn(&S) -> Score>(initial_state: S, heuristic: H) -> Option<(Vec<S::Edge>, SolveStats)> {
    // "Reverse Edges". A hashmap from state to previous state + edge
    let mut redges = HashMap::new();

    // a priority queue, implemented using the standard library's binary heap.
    let mut work_queue = BinaryHeap::<Node<S>>::new();
    work_queue.push(Node{
        score: 0,
        cost: 0,
        state: initial_state,
        prev: None,
    });
    let mut work_count = 0;
    while let Some(work) = work_queue.pop() {
        // Break the fields of the "work" node out into variables old_state, old_cost, and old_path.
        // These are from the node we're coming from
        match work {
            Node { state: old_state, cost: old_cost, score: _, prev } => {
                // If we're the first to reach "old_state" (which is the current work item)
                // then the old_state's previous edge is the fastest route there
                // so we save the old_state's previous edge and previous game state
                if let Some(prev) = prev {
                    redges.entry(old_state.clone()).or_insert(prev);
                }
                if old_state.is_solved() {
                    let mut path = vec![];
                    let mut state = old_state;
                    while let Some((prev_state, prev_edge)) = redges.remove(&state) {
                        path.push(prev_edge);
                        state = prev_state;
                    }
                    // Reverse the path from destination to initial_state to be forwards
                    path.reverse();
                    let stats = SolveStats{
                        path_len: path.len(),
                        work_count,
                        work_queue_len: work_queue.len()
                    };
                    return Some((path, stats));
                }
                for (state, cost, edge) in old_state.clone().iter_successors() {
                    let cost = cost + old_cost;
                    let node = Node{
                        score: heuristic(&state) + cost,
                        cost,
                        state,
                        prev: Some((old_state.clone(), edge))
                    };
                    work_queue.push(node);
                }
            }
        }
        work_count += 1;
    }
    None
}

// manual trait implementations, to make it so Node's orderings only depend on the score field.
// (Oh, and I'm also flipping the order
impl<S: State> Ord for Node<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Normal ordering:
        // self.score.cmp(&other.score)
        // Flipped, as we want low-cost nodes first
        other.score.cmp(&self.score)
    }
}

impl<S: State> PartialOrd for Node<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: State> PartialEq for Node<S> {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl<S: State> Eq for Node<S> {}

/*
impl<S: State> fmt::Debug for Node<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.state.fmt(f)
    }
}
*/

