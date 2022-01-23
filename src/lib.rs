//! https://www.baeldung.com/java-monte-carlo-tree-search

#![allow(dead_code)]

use bumpalo::collections::Vec;
use bumpalo::Bump;

use rand::Rng;

mod basic_search;

pub trait GameState {
    fn points(&self) -> i32;

    fn next(&self) -> &[Self]
    where
        Self: Sized;

    fn is_finished(&self) -> bool;
}

type Tree<'tree, S> = Node<'tree, S>;

struct Node<'tree, S> {
    state: S,
    visited: u64,
    won: u64,
    parent: Option<&'tree Node<'tree, S>>,
    children: Vec<'tree, Node<'tree, S>>,
}

impl<'tree, S: GameState> Node<'tree, S> {
    fn new(state: S, alloc: &'tree Bump) -> Node<S> {
        Node {
            state,
            visited: 0,
            won: 0,
            parent: None,
            children: Vec::new_in(alloc),
        }
    }

    fn random_child(&self) -> &Self {
        &self.children[rand::thread_rng().gen_range(0..self.children.len())]
    }

    fn into_child_with_max_score(self) -> Self {
        todo!()
    }
}

mod mcts {
    use crate::{GameState, Node};
    use bumpalo::Bump;

    const MAX_TRIES: u64 = 10000;

    pub fn find_next_move<S: GameState>(current_state: S) -> S {
        let alloc = Bump::new();

        let root_node = Node::new(current_state, &alloc);

        for _ in 0..MAX_TRIES {
            let promising_node = select_promising_node(&root_node);

            if !promising_node.state.is_finished() {
                expand_node(promising_node);
            }

            let node_to_explore = if !node_to_explore.children.is_empty() {
                promising_node.random_child()
            } else {
                promising_node
            };

            let playout_result = simulate_random_playout(node_to_explore);
            back_propagation(node_to_explore, playout_result);
        }

        let winner_node = root_node.into_child_with_max_score();

        winner_node.state
    }

    fn select_promising_node<'tree, S>(node: &'tree Node<'_, S>) -> &'tree Node<'tree, S> {
        todo!()
    }

    fn expand_node<S>(node: &Node<S>) {
        todo!("next")
    }

    fn simulate_random_playout<S>(node: &Node<'_, S>) -> u64 {
        todo!()
    }

    fn back_propagation<S>(node: &Node<'_, S>, playout_result: u64) {
        todo!()
    }

    mod uct {
        use crate::Node;

        pub fn uct(total_visit: u64, win_score: u64, node_visit: u64) -> u64 {
            if node_visit == 0 {
                return u64::MAX;
            }

            let num = (win_score / node_visit) as f64
                + std::f64::consts::SQRT_2
                    * f64::sqrt((total_visit as f64).ln() / node_visit as f64);

            num as u64
        }

        pub fn find_best_node_with_uct<S>(node: &Node<S>) -> Option<&Node<S>> {
            let parent_visit_count = node.visited;

            node.children
                .iter()
                .max_by_key(|n| uct(parent_visit_count, n.won, n.visited))
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn t() {}
}
