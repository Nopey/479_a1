//! A* Search Algorithm implementation
//!
//! Contains no domain-specific knowledge about the ball-game.
use std::collections::{BinaryHeap, HashSet};
use std::cmp::Ordering;
use std::{fmt,fmt::Debug};
use std::hash::Hash;

/// An interface to expose a game's successor function to the search alg.
///
/// Implemented by struct Game in game.rs
pub trait State: Sized + Hash + Eq + Clone + Debug {
    /// The search algorithm will return the path as a vector of Edges.
    type Edge: Clone + Clone + Debug;
    /// An iterator over the neighboring states, their cost, and the 'Edge' to return if this is used as the solution Path.
    type Iter: Iterator<Item = (Self, Cost, Self::Edge)>;
    /// Return an iterator over the neighboring states 
    fn iter_successors(self) -> Self::Iter;
    /// Take an edge, if that edge exists.
    fn try_edge(&self, edge: &Self::Edge) -> Option<Self>;

    fn is_solved(&self) -> bool;
}

/// A typedef for the integer I'm using to keep track of cost
pub type Cost = i32;

/// A path to be considered, ordered by astar_costs.
struct Node<S: State> {
    /// cost + heuristic's predicted future cost
    astar_cost: Cost,
    /// Path cost to this path finding node
    cost: Cost,
    /// edges leading to this state from the initial_state
    path: Vec<S::Edge>,
}

/// Statistics about how difficult a solution was to find
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

/// A generic implementation of A*, which takes an initial state and a heuristic.
///
/// Returns `None` if no solution is found, or `Some((Path, Stats))` otherwise
pub fn solve<S: State, H: Fn(&S) -> Cost>(initial_state: S, heuristic: H) -> Option<(Vec<S::Edge>, SolveStats)> {
    // the set of all states we've visited
    let mut visited = HashSet::new();
    // a priority queue, implemented using the standard library's binary heap.
    let mut work_queue = BinaryHeap::<Node<S>>::new();
    // Push our starting node
    work_queue.push(Node{
        astar_cost: 0,
        cost: 0,
        path: vec![]
    });

    let mut last_cost = 0;
    // Loop over the work queue. Nodes with the least cost will be considered first.
    while let Some(work) = work_queue.pop() {
        // a useful assert I discovered all too late in development
        debug_assert!(last_cost <= work.astar_cost, "INADMISSABLE {} -> {}\n{:?}", last_cost, work.astar_cost, work.path);
        last_cost = work.astar_cost;

        // Break the fields of the "work" node out into variables cost and path while ignoring field 'astar_cost'
        // These are from the node we're coming from
        let Node { cost, astar_cost: _, path } = work;

        // Follow the edges from the initial state to the state described by `work`.
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
                astar_cost: heuristic(&next_state) + cost,
                cost,
                path: new_path
            };
            work_queue.push(node);
        }
    }
    None
}

// manual trait implementations, to make it so Node's orderings only depend on the astar_cost field.
impl<S: State> Ord for Node<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        // NOTE: In std's BinaryHeap the largest Node will be popped first, so this
        // code makes it so Node A > Node B if Node A's cost < Node B's cost
        other.astar_cost.cmp(&self.astar_cost)
    }
}

impl<S: State> PartialOrd for Node<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: State> PartialEq for Node<S> {
    fn eq(&self, other: &Self) -> bool {
        self.astar_cost == other.astar_cost
    }
}

impl<S: State> Eq for Node<S> {}
