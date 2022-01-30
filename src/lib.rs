//! https://www.baeldung.com/java-monte-carlo-tree-search

mod basic_search;

pub use mcts::find_next_move;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Status<P> {
    InProgress,
    Draw,
    Winner(P),
}

pub trait PlayerState: Eq + Copy {
    fn next(self) -> Self;
}

pub trait GameState: Clone + std::fmt::Debug {
    type Player: PlayerState + std::fmt::Debug;
    type NextStates: ExactSizeIterator<Item = Self>;

    fn next_states(&self) -> Self::NextStates;

    fn status(&self) -> Status<Self::Player>;

    fn toggle_player(&mut self);

    fn next_random_play(&mut self);
}

mod mcts {
    use crate::{GameState, PlayerState, Status};
    use bumpalo::Bump;
    use rand::Rng;
    use std::cell::Cell;

    #[derive(Debug, Clone)]
    struct Node<'tree, S: GameState> {
        state: S,
        // todo: don't have this field and let the GameState handle this all
        player: S::Player,
        visited: Cell<u32>,
        score: Cell<i32>,
        parent: Option<&'tree Node<'tree, S>>,
        children: Cell<&'tree [Node<'tree, S>]>,
    }

    impl<'tree, S: GameState> Node<'tree, S> {
        fn new(state: S, player: S::Player, alloc: &'tree Bump) -> Node<'tree, S> {
            Self {
                state,
                player,
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

    const MAX_TRIES: u64 = 10_000;

    pub fn find_next_move<S: GameState>(current_state: S, own_player: S::Player) -> S {
        let alloc = Bump::new();
        let opponent = own_player.next();

        let root_node = alloc.alloc(Node::new(current_state, opponent, &alloc));

        for _ in 0..MAX_TRIES {
            // Phase 1 - Selection
            let promising_node = select_promising_node(root_node);

            // Phase 2 - Expansion
            if promising_node.state.status() == Status::InProgress {
                expand_node(&alloc, promising_node);
            }

            // Phase 3 - Simulation
            let promising_node = if !promising_node.children.get().is_empty() {
                promising_node.random_child()
            } else {
                promising_node
            };
            let playout_result = simulate_random_playout(promising_node, opponent);

            // Phase 4 - Update
            back_propagation(promising_node, playout_result);
        }

        let winner_node = root_node.child_with_max_score();

        let node = winner_node.unwrap();
        node.state.clone()
    }

    fn select_promising_node<'tree, S: GameState>(
        root_node: &'tree Node<'tree, S>,
    ) -> &'tree Node<'tree, S> {
        let mut node = root_node;

        while !node.children.get().is_empty() {
            node = uct::find_best_node_with_uct(node).unwrap()
        }

        node
    }

    fn expand_node<'tree, S: GameState>(alloc: &'tree Bump, node: &'tree Node<'tree, S>) {
        let possible_states = node.state.next_states();

        let new_nodes = possible_states.map(|state| Node {
            state,
            player: node.player.next(),
            visited: Cell::new(0),
            score: Cell::new(0),
            parent: Some(node),
            children: Cell::new(alloc.alloc([])),
        });

        let children = alloc.alloc_slice_fill_iter(new_nodes);

        node.children.set(children);
    }

    fn back_propagation<S: GameState>(node: &Node<'_, S>, resulting_status: Status<S::Player>) {
        let mut temp_node = Some(node);

        while let Some(node) = temp_node {
            node.visited.set(node.visited.get() + 1);

            if node.state.status() == resulting_status {
                node.score.set(node.score.get() + 1);
            }

            temp_node = node.parent;
        }
    }

    fn simulate_random_playout<S: GameState>(
        node: &Node<'_, S>,
        opponent: S::Player,
    ) -> Status<S::Player> {
        let mut state = node.state.clone();

        let mut board_status = state.status();

        if board_status == Status::Winner(opponent) {
            if let Some(parent) = node.parent {
                parent.score.set(i32::MIN)
            }
            return board_status;
        }

        while board_status == Status::InProgress {
            state.toggle_player();
            state.next_random_play();
            board_status = state.status();
        }

        board_status
    }

    mod uct {
        use crate::mcts::Node;
        use crate::GameState;

        pub fn uct(total_visit: u32, win_score: i32, node_visit: i32) -> u32 {
            if node_visit == 0 {
                return u32::MAX;
            }

            let num = (win_score / node_visit) as f64
                + std::f64::consts::SQRT_2
                    * f64::sqrt((total_visit as f64).ln() / node_visit as f64);

            num as u32
        }

        pub(super) fn find_best_node_with_uct<'tree, S: GameState>(
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

pub mod tic_tac_toe {
    use crate::{GameState, PlayerState, Status};
    use rand::Rng;
    use std::fmt::{Display, Formatter, Write};

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum Player {
        O,
        X,
    }

    impl crate::PlayerState for Player {
        fn next(self) -> Self {
            match self {
                Self::O => Self::X,
                Self::X => Self::O,
            }
        }
    }

    #[derive(Debug, Copy, Clone)]
    enum State {
        Empty,
        X,
        O,
    }

    impl Display for State {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                State::Empty => f.write_char(' '),
                State::X => f.write_char('X'),
                State::O => f.write_char('O'),
            }
        }
    }

    impl From<Player> for State {
        fn from(player: Player) -> State {
            match player {
                Player::O => State::O,
                Player::X => State::X,
            }
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Board {
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

        fn free_fields(&self) -> usize {
            self.board
                .iter()
                .filter(|field| matches!(field, State::Empty))
                .count()
        }
    }

    impl GameState for Board {
        type Player = Player;

        type NextStates = std::vec::IntoIter<Self>;

        fn next_states(&self) -> Self::NextStates {
            let state_iter = self
                .board
                .iter()
                .enumerate()
                .filter(|(_, field)| matches!(field, State::Empty))
                .map(|(i, _)| {
                    let mut new_state = *self;

                    new_state.active_player = self.active_player.next();
                    new_state.board[i] = new_state.active_player.into();

                    new_state
                })
                .collect::<Vec<_>>()
                .into_iter();

            state_iter
        }

        fn status(&self) -> Status<Player> {
            if self.free_fields() == 0 {
                return Status::Draw;
            }

            let all_checks = [
                // rows
                [0, 1, 2],
                [3, 4, 5],
                [6, 7, 8],
                // columns
                [0, 3, 6],
                [1, 4, 7],
                [2, 5, 8],
                // diagonals
                [0, 4, 8],
                [2, 4, 6],
            ];

            for check in all_checks {
                match check.map(|i| &self.board[i]) {
                    [State::X, State::X, State::X] => return Status::Winner(Player::X),
                    [State::O, State::O, State::O] => return Status::Winner(Player::O),
                    _ => {}
                }
            }

            Status::InProgress
        }

        fn toggle_player(&mut self) {
            self.active_player = self.active_player.next();
        }

        fn next_random_play(&mut self) {
            let free_fields = self.free_fields();
            let random_field = rand::thread_rng().gen_range(0..free_fields);

            let (field_idx, _) = self
                .board
                .iter()
                .enumerate()
                .filter(|(_, field)| matches!(field, State::Empty))
                .nth(random_field)
                .unwrap();

            self.board[field_idx] = self.active_player.into();
        }
    }

    impl Display for Board {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let b = &self.board;

            write!(
                f,
                "    a   b   c
  ╭───┬───┬───╮
1 │ {} │ {} │ {} │
  ├───┼───┼───┤
2 │ {} │ {} │ {} │
  ├───┼───┼───┤
3 │ {} │ {} │ {} │
  ╰───┴───┴───╯",
                b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8]
            )
        }
    }

    pub use run::main;

    mod run {
        use super::{Board, Player};
        use crate::tic_tac_toe::State;
        use crate::{mcts, GameState, Status};
        use std::io::Write;

        const PLAYING_PLAYER: Player = Player::O;
        const AI_PLAYER: Player = Player::X;

        pub fn main() {
            let mut board = Board::new(PLAYING_PLAYER);

            let winner = loop {
                println!("{}", board);
                process_player_input(&mut board);

                if let Some(result) = is_finished(&board) {
                    break result;
                }

                let ai_play = mcts::find_next_move(board, AI_PLAYER);
                board = ai_play;

                if let Some(result) = is_finished(&board) {
                    break result;
                }
            };

            println!("{}", board);
            match winner {
                Some(player) => println!("player {} won!", State::from(player)),
                None => println!("draw!"),
            }
        }

        fn is_finished(board: &Board) -> Option<Option<Player>> {
            if let Status::Winner(winner) = board.status() {
                return Some(Some(winner));
            }

            if board.free_fields() == 0 {
                return Some(None);
            }

            None
        }

        fn process_player_input(board: &mut Board) {
            loop {
                let player_input = get_player_pos();

                match board.board[player_input] {
                    State::Empty => {
                        board.board[player_input] = PLAYING_PLAYER.into();
                        return;
                    }
                    _ => {
                        println!("Field is already taken.")
                    }
                }
            }
        }

        fn get_player_pos() -> usize {
            loop {
                print!("your move (xy): ");
                std::io::stdout().flush().unwrap();

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();

                let mut chars = input.chars();

                match [chars.next(), chars.next()] {
                    [Some(x_char @ ('a' | 'b' | 'c')), Some(y_char @ ('1' | '2' | '3'))] => {
                        let x = (x_char as u8) - b'a';
                        let y = (y_char as u8) - b'1';

                        return (x + (3 * y)) as usize;
                    }
                    _ => eprintln!("Invalid input: {}", input),
                }
            }
        }
    }
}
