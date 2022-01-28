//! https://www.baeldung.com/java-monte-carlo-tree-search

mod basic_search;

pub use mcts::find_next_move;

pub trait GameState: Clone {
    type Player: Eq + Copy;

    fn next_states(&self) -> Box<dyn ExactSizeIterator<Item = Self>>;

    fn player_won(&self) -> Option<Self::Player>;

    fn next_random_play(&mut self);
}

mod mcts {
    use crate::GameState;
    use bumpalo::Bump;
    use rand::Rng;
    use std::cell::Cell;

    #[derive(Clone)]
    struct Node<'tree, S> {
        state: S,
        visited: Cell<u32>,
        score: Cell<i32>,
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

    pub fn find_next_move<S: GameState>(current_state: S, opponent: S::Player) -> S {
        let alloc = Bump::new();

        let root_node = alloc.alloc(Node::new(current_state, &alloc));

        for _ in 0..MAX_TRIES {
            let promising_node = select_promising_node(root_node);

            if promising_node.state.player_won() == None {
                expand_node(&alloc, promising_node);
            }

            if !promising_node.children.get().is_empty() {
                let child = promising_node.random_child();
                let playout_result = simulate_random_playout(child, opponent);
                back_propagation(child, playout_result);
            } else {
                let playout_result = simulate_random_playout(promising_node, opponent);
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

    fn back_propagation<S: GameState>(node: &Node<'_, S>, player_won: S::Player) {
        let mut temp_node = Some(node);

        while let Some(node) = temp_node {
            node.visited.set(node.visited.get() + 1);

            if node.state.player_won() == Some(player_won) {
                node.score.set(node.score.get() + 1);
            }

            temp_node = node.parent;
        }
    }

    fn simulate_random_playout<S: GameState>(node: &Node<'_, S>, opponent: S::Player) -> S::Player {
        let mut state = node.state.clone();

        let mut board_status = state.player_won();

        if board_status == Some(opponent) {
            if let Some(parent) = node.parent {
                parent.score.set(i32::MIN)
            }
            return opponent;
        }

        loop {
            match board_status {
                None => {
                    state.next_random_play();
                    board_status = state.player_won();
                }
                Some(player) => return player,
            }
        }
    }

    mod uct {
        use crate::mcts::Node;

        pub fn uct(total_visit: u32, win_score: i32, node_visit: i32) -> u32 {
            if node_visit == 0 {
                return u32::MAX;
            }

            let num = (win_score / node_visit) as f64
                + std::f64::consts::SQRT_2
                    * f64::sqrt((total_visit as f64).ln() / node_visit as f64);

            num as u32
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

mod tic_tac_toe {
    use crate::GameState;

    #[derive(Copy, Clone, Eq, PartialEq)]
    enum Player {
        O,
        X,
    }

    #[derive(Copy, Clone)]
    enum State {
        Empty,
        X,
        O,
    }

    #[derive(Copy, Clone)]
    struct Board {
        active_player: Player,
        board: [State; 9],
    }

    impl Board {
        pub fn new(starter: Player) -> Self {
            Self {
                active_player: starter,
                board: [State::Empty; 9],
            }
        }
    }

    impl GameState for Board {
        type Player = Player;

        fn next_states(&self) -> Box<dyn ExactSizeIterator<Item = Self>> {
            todo!()
        }

        fn player_won(&self) -> Option<Player> {
            todo!()
        }

        fn next_random_play(&mut self) {
            todo!()
        }
    }
}
