//! https://www.baeldung.com/java-monte-carlo-tree-search

mod basic_search;

pub use mcts::find_next_move;

pub trait GameState {
    fn points(&self) -> i32;

    fn next_states(&self) -> Box<dyn ExactSizeIterator<Item = Self>>;

    fn is_finished(&self) -> bool;
}

mod mcts {
    use crate::GameState;
    use bumpalo::Bump;
    use rand::Rng;
    use std::cell::Cell;

    #[derive(Clone)]
    struct Node<'tree, S> {
        state: S,
        visited: Cell<u64>,
        score: Cell<u64>,
        parent: Option<&'tree Node<'tree, S>>,
        children: Cell<&'tree [Node<'tree, S>]>,
    }

    impl<'tree, S> Node<'tree, S> {
        fn new(state: S, alloc: &'tree Bump) -> Node<S> {
            Node {
                state,
                visited: Cell::new(0),
                score: Cell::new(0),
                parent: None,
                children: Cell::new(alloc.alloc([])),
            }
        }

        fn random_child(&self) -> &Self {
            let children = self.children.get();
            let random_index = rand::thread_rng().gen_range(0..children.len());

            &children[random_index]
        }

        fn child_with_max_score(&self) -> Option<&Self> {
            self.children
                .get()
                .iter()
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

            if !promising_node.children.get().is_empty() {
                let child = promising_node.random_child();
                let playout_result = simulate_random_playout(child);
                back_propagation(child, playout_result);
            } else {
                let playout_result = simulate_random_playout(promising_node);
                back_propagation(promising_node, playout_result);
            };
        }

        let winner_node = root_node.child_with_max_score();

        let node = winner_node.unwrap();
        node.state.clone()
    }

    fn select_promising_node<'tree, S>(root_node: &'tree Node<'tree, S>) -> &'tree Node<'tree, S> {
        let mut node = root_node;

        while !node.children.get().is_empty() {
            node = uct::find_best_node_with_uct(root_node).unwrap()
        }

        node
    }

    fn expand_node<'tree, S: GameState>(alloc: &'tree Bump, node: &'tree Node<'tree, S>) {
        let possible_states = node.state.next_states();

        let new_nodes = possible_states.map(|state| Node {
            state,
            visited: Cell::new(0),
            score: Cell::new(0),
            parent: Some(node),
            children: Cell::new(alloc.alloc([])),
        });

        let children = alloc.alloc_slice_fill_iter(new_nodes);

        node.children.set(children);
    }

    fn back_propagation<S>(node: &Node<'_, S>, _playout_result: u64) {
        let mut temp_node = Some(node);

        while let Some(node) = temp_node {
            // todo increment visit
            // todo increment win count if we won
            temp_node = node.parent;
        }
    }

    fn simulate_random_playout<S>(_node: &Node<'_, S>) -> u64 {
        /*
        Node tempNode = new Node(node);
        State tempState = tempNode.getState();
        int boardStatus = tempState.getBoard().checkStatus();
        if (boardStatus == opponent) {
            tempNode.getParent().getState().setWinScore(Integer.MIN_VALUE);
            return boardStatus;
        }
        while (boardStatus == Board.IN_PROGRESS) {
            tempState.togglePlayer();
            tempState.randomPlay();
            boardStatus = tempState.getBoard().checkStatus();
        }
        return boardStatus;
             */
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
                .get()
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
