//! https://www.baeldung.com/java-monte-carlo-tree-search

#![allow(dead_code)]

use bumpalo::collections::Vec;
use bumpalo::Bump;
use std::cell::RefCell;

use rand::{random, Rng};

mod basic_search;

pub trait GameState {
    fn points(&self) -> i32;

    fn next(&self) -> &[Self]
    where
        Self: Sized;

    fn is_finished(&self) -> bool;
}

type Tree<'tree, S> = RefCell<Node<'tree, S>>;

#[derive(Clone)]
struct Node<'tree, S> {
    state: S,
    visited: u64,
    score: u64,
    parent: Option<&'tree RefNode<'tree, S>>,
    children: Vec<'tree, RefNode<'tree, S>>,
}

type RefNode<'tree, S> = RefCell<Node<'tree, S>>;

impl<'tree, S> Node<'tree, S> {
    fn new(state: S, alloc: &'tree Bump) -> Node<S> {
        Node {
            state,
            visited: 0,
            score: 0,
            parent: None,
            children: Vec::new_in(alloc),
        }
    }

    fn random_child(&self) -> &RefCell<Self> {
        let random_index = rand::thread_rng().gen_range(0..self.children.len());

        &self.children[random_index]
    }

    fn into_child_with_max_score(self) -> Option<RefCell<Self>> {
        self.children
            .into_iter()
            .max_by_key(|node| node.borrow().score)
    }
}

mod mcts {
    use crate::{GameState, Node, RefNode};
    use bumpalo::Bump;
    use std::cell::{Ref, RefCell};

    const MAX_TRIES: u64 = 10000;

    pub fn find_next_move<S: GameState + Clone>(current_state: S) -> S {
        let alloc = Bump::new();

        let root_node = alloc.alloc(RefCell::new(Node::new(current_state, &alloc)));

        for _ in 0..MAX_TRIES {
            let promising_node = select_promising_node(&root_node);

            if !promising_node.borrow().state.is_finished() {
                expand_node(&promising_node);
            }

            if !promising_node.borrow().children.is_empty() {
                let promising_node = promising_node.borrow();
                let child = promising_node.random_child();
                let playout_result = simulate_random_playout(&child);
                back_propagation(&child, playout_result);
            } else {
                let playout_result = simulate_random_playout(&promising_node);
                back_propagation(&promising_node, playout_result);
            };
        }

        let winner_node = root_node.clone().into_inner().into_child_with_max_score();

        let state = winner_node.unwrap().into_inner().state;
        state
    }

    fn select_promising_node<'cell, 'tree, S>(
        root_node: &'cell RefCell<Node<'tree, S>>,
    ) -> &'cell RefNode<'tree, S> {
        let mut node = root_node;

        while node.borrow().children.len() != 0 {
            node = uct::find_best_node_with_uct(&root_node).unwrap()
        }

        node
    }

    fn expand_node<S>(_node: &RefNode<S>) {
        todo!("next")
    }

    fn simulate_random_playout<S>(_node: &RefNode<'_, S>) -> u64 {
        todo!()
    }

    fn back_propagation<S>(_node: &RefNode<'_, S>, _playout_result: u64) {
        todo!()
    }

    mod uct {
        use crate::mcts::RefNode;
        use crate::Node;
        use std::cell::Ref;

        pub fn uct(total_visit: u64, win_score: u64, node_visit: u64) -> u64 {
            if node_visit == 0 {
                return u64::MAX;
            }

            let num = (win_score / node_visit) as f64
                + std::f64::consts::SQRT_2
                    * f64::sqrt((total_visit as f64).ln() / node_visit as f64);

            num as u64
        }

        pub fn find_best_node_with_uct<'cell, 'tree, S>(
            node: Ref<'cell, Node<'tree, S>>,
        ) -> Option<&'cell RefNode<'tree, S>> {
            let parent_visit_count = node.visited;

            node.children.iter().max_by_key(|n| {
                let n = n.borrow();
                uct(parent_visit_count, n.score, n.visited)
            })
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn t() {}
}
