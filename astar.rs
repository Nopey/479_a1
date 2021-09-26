//! A somewhat generic implementation of the A* search algorithm.
//!
//! Contains no domain-specific knowledge about the ball-game.
use std::collections::{BinaryHeap, HashSet};
use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;

/// An interface to expose a game's successor function to the search alg.
///
/// Implemented by struct Game in game.rs
pub trait State: Sized + Hash + Eq + Clone {
    /// The search algorithm will return the path as a vector of Edges.
    type Edge: Clone + Clone;
    /// An iterator over the neighboring states, their cost, and the 'Edge' to return if this is used as the solution Path.
    type Iter: Iterator<Item = (Self, Score, Self::Edge)>;
    /// Return an iterator over the neighboring states 
    fn iter_successors(self) -> Self::Iter;
    /// Take an edge, if that edge exists.
    fn try_edge(&self, edge: &Self::Edge) -> Option<Self>;

    fn is_solved(&self) -> bool;
}

/// A typedef for the integer I'm using to keep track of score and cost
//TODO: Rename Score to `Cost`
pub type Score = i32;

/// A simple wrapper around a State and a heuristic's score, that implements the Ord (Ordering) trait,
/// which allows the states to be sorted by their score in a binary heap.
struct Node<S: State> {
    /// Heuristic's score
    score: Score,
    /// Path cost to this path finding node
    cost: Score,
    /// edges leading to this state from the initial_state
    path: Vec<S::Edge>,
}

/// Statistics about how difficult the problem was to solve
pub struct SolveStats {
    path_len: usize,
    visited_len: usize,
    work_queue_len: usize,
}
impl fmt::Display for SolveStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Solved for {} long path by visiting {} nodes. work queue len: {}", self.path_len, self.visited_len, self.work_queue_len)
    }
}

/// A generic implementation of A*, which takes an initial state and heuristic.
///
/// Returns `None` if no solution is found, or `Some((Path, Stats))` otherwise
pub fn solve<S: State, H: Fn(&S) -> Score>(initial_state: S, heuristic: H) -> Option<(Vec<S::Edge>, SolveStats)> {
    // the set of all states we've visited
    let mut visited = HashSet::new();
    // a priority queue, implemented using the standard library's binary heap.
    let mut work_queue = BinaryHeap::<Node<S>>::new();
    // Push our starting node
    work_queue.push(Node{
        score: 0,
        cost: 0,
        path: vec![]
    });

    // Loop over the work queue. Nodes with the least cost will be considered first.
    while let Some(work) = work_queue.pop() {
        // Break the fields of the "work" node out into variables cost and path while ignoring field 'score'
        // These are from the node we're coming from
        let Node { cost, score: _, path } = work;
        let state = {
            let mut state = initial_state.clone();
            for edge in &path {
                state = state.try_edge(edge).unwrap();
            }
            state
        };
        if state.is_solved() {
            let stats = SolveStats{
                path_len: path.len(),
                work_queue_len: work_queue.len(),
                visited_len: visited.len()
            };
            return Some((path, stats));
        }
        // If we're the first to reach state
        // then the state's previous edge is the fastest route there
        if visited.get(&state).is_none() {
            visited.insert(state.clone());
        }else{
            // already visited node, skip any further work
            continue;
        }
        for (next_state, edge_cost, edge) in state.clone().iter_successors() {
            let cost = cost + edge_cost;
            let mut new_path = Vec::with_capacity(path.len());
            new_path.extend(path.iter().cloned());
            new_path.push(edge.clone());
            let node = Node{
                score: heuristic(&next_state) + cost,
                cost,
                path: new_path
            };
            work_queue.push(node);
        }
    }
    None
}

// manual trait implementations, to make it so Node's orderings only depend on the score field.
impl<S: State> Ord for Node<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        // NOTE: In std's BinaryHeap the largest Node will be popped first, so this
        // code makes it so Node A > Node B if Node A's cost < Node B's cost
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
