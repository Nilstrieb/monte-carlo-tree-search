//! https://www.baeldung.com/java-monte-carlo-tree-search

mod basic_search;

pub use mcts::find_next_move;

pub trait GameState {
    fn points(&self) -> i32;

    fn next_states(&self) -> Box<dyn Iterator<Item = Self>>;

    fn is_finished(&self) -> bool;
}

mod mcts {
    use crate::GameState;
    use bumpalo::collections::Vec as BumpVec;
    use bumpalo::Bump;
    use rand::Rng;
    use std::cell::Cell;

    #[derive(Clone)]
    struct Node<'tree, S> {
        state: S,
        visited: Cell<u64>,
        score: Cell<u64>,
        parent: Option<&'tree Node<'tree, S>>,
        children: BumpVec<'tree, Node<'tree, S>>,
    }

    impl<'tree, S> Node<'tree, S> {
        fn new(state: S, alloc: &'tree Bump) -> Node<S> {
            Node {
                state,
                visited: Cell::new(0),
                score: Cell::new(0),
                parent: None,
                children: BumpVec::new_in(alloc),
            }
        }

        fn random_child(&self) -> &Self {
            let random_index = rand::thread_rng().gen_range(0..self.children.len());

            &self.children[random_index]
        }

        fn into_child_with_max_score(self) -> Option<Self> {
            self.children
                .into_iter()
                .max_by_key(|node| node.score.get())
        }
    }

    const MAX_TRIES: u64 = 10000;

    pub fn find_next_move<S: GameState + Clone>(current_state: S) -> S {
        let alloc = Bump::new();

        let root_node = alloc.alloc(Node::new(current_state, &alloc));

        for _ in 0..MAX_TRIES {
            let promising_node = select_promising_node(root_node);

            if !promising_node.state.is_finished() {
                expand_node(&alloc, promising_node);
            }

            if !promising_node.children.is_empty() {
                let child = promising_node.random_child();
                let playout_result = simulate_random_playout(child);
                back_propagation(child, playout_result);
            } else {
                let playout_result = simulate_random_playout(promising_node);
                back_propagation(promising_node, playout_result);
            };
        }

        let winner_node = root_node.clone().into_child_with_max_score();

        let node = winner_node.unwrap();
        node.state
    }

    fn select_promising_node<'tree, S>(root_node: &'tree Node<'tree, S>) -> &'tree Node<'tree, S> {
        let mut node = root_node;

        while !node.children.is_empty() {
            node = uct::find_best_node_with_uct(root_node).unwrap()
        }

        node
    }

    fn expand_node<S: GameState>(alloc: &Bump, node: &Node<'_, S>) {
        /*
               List<State> possibleStates = node.getState().getAllPossibleStates();
        possibleStates.forEach(state -> {
            Node newNode = new Node(state);
            newNode.setParent(node);
            newNode.getState().setPlayerNo(node.getState().getOpponent());
            node.getChildArray().add(newNode);
        });
             */
        let possible_states = node.state.next_states();
        for state in possible_states {
            let child = Node {
                state,
                visited: Cell::new(0),
                score: Cell::new(0),
                parent: Some(node),
                children: BumpVec::new_in(alloc),
            };
        }
    }

    fn simulate_random_playout<S>(_node: &Node<'_, S>) -> u64 {
        todo!()
    }

    fn back_propagation<S>(_node: &Node<'_, S>, _playout_result: u64) {
        todo!()
    }

    mod uct {
        use crate::mcts::Node;

        pub fn uct(total_visit: u64, win_score: u64, node_visit: u64) -> u64 {
            if node_visit == 0 {
                return u64::MAX;
            }

            let num = (win_score / node_visit) as f64
                + std::f64::consts::SQRT_2
                    * f64::sqrt((total_visit as f64).ln() / node_visit as f64);

            num as u64
        }

        pub(super) fn find_best_node_with_uct<'tree, S>(
            node: &'tree Node<'tree, S>,
        ) -> Option<&'tree Node<'tree, S>> {
            let parent_visit_count = node.visited.get();

            node.children
                .iter()
                .max_by_key(|n| uct(parent_visit_count, n.score.get(), n.score.get()))
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn t() {}
}
