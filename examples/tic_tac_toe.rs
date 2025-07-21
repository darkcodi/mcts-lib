extern crate mcts_lib;

use mcts_lib::board::{Board, GameOutcome, Player};
use mcts_lib::mcts::MonteCarloTreeSearch;
use mcts_lib::random::StandardRandomGenerator;
use std::fmt::Debug;

fn main() {
    // Create a new Tic-Tac-Toe board
    let board = TicTacToeBoard::default();

    // Create a new MCTS search instance
    let mut mcts = MonteCarloTreeSearch::builder(board)
        .with_alpha_beta_pruning(false)
        .with_random_generator(StandardRandomGenerator::default())
        .build();

    // Run the search for 20,000 iterations
    mcts.iterate_n_times(20000);

    // Print the chances
    let tree = mcts.get_tree();
    let root = mcts.get_root();
    for node_id in root.children() {
        let node = tree.get(node_id).unwrap();
        println!(
            "Move: {:?} = {:.2?}%",
            node.data().prev_move,
            node.data().wins_rate() * 100.0
        );
    }

    // Get the most promising move
    let best_move_node = mcts.get_most_perspective_move();
    let best_move = best_move_node.data().prev_move;

    println!("The best move is: {:?}", best_move);
}

/// An implementation of the `Board` trait for the game of Tic-Tac-Toe.
///
/// The board is represented by a 9-element array, where each element corresponds to a cell.
/// A move is represented by a `u8` from 0 to 8.
pub struct TicTacToeBoard {
    root_player: TTTPlayer,
    current_player: TTTPlayer,
    field: [Option<TTTPlayer>; 9],
    outcome: GameOutcome,
}

impl TicTacToeBoard {
    fn new(root_player: TTTPlayer) -> Self {
        Self {
            root_player,
            current_player: TTTPlayer::X,
            field: [None; 9],
            outcome: GameOutcome::InProgress,
        }
    }
}

impl Default for TicTacToeBoard {
    /// Creates a new Tic-Tac-Toe board with player 'X' starting.
    fn default() -> Self {
        TicTacToeBoard::new(TTTPlayer::X)
    }
}

impl Clone for TicTacToeBoard {
    fn clone(&self) -> Self {
        let mut copied_field = [None; 9];
        copied_field.copy_from_slice(&self.field);
        Self {
            root_player: self.root_player,
            current_player: self.current_player,
            field: copied_field,
            outcome: self.outcome,
        }
    }
}

impl Board for TicTacToeBoard {
    type Move = u8;

    fn get_current_player(&self) -> Player {
        match self.current_player == self.root_player {
            true => Player::Me,
            false => Player::Other,
        }
    }

    fn get_outcome(&self) -> GameOutcome {
        if self.field[0].is_some()
            && (self.field[0] == self.field[1] && self.field[0] == self.field[2]
                || self.field[0] == self.field[3] && self.field[0] == self.field[6])
        {
            return if self.field[0].unwrap() == self.root_player {
                GameOutcome::Win
            } else {
                GameOutcome::Lose
            };
        }

        if self.field[8].is_some()
            && (self.field[8] == self.field[2] && self.field[8] == self.field[5]
                || self.field[8] == self.field[6] && self.field[8] == self.field[7])
        {
            return if self.field[8].unwrap() == self.root_player {
                GameOutcome::Win
            } else {
                GameOutcome::Lose
            };
        }

        if self.field[4].is_some()
            && (self.field[4] == self.field[1] && self.field[4] == self.field[7]
                || self.field[4] == self.field[3] && self.field[4] == self.field[5]
                || self.field[4] == self.field[0] && self.field[4] == self.field[8]
                || self.field[4] == self.field[2] && self.field[4] == self.field[6])
        {
            return if self.field[4].unwrap() == self.root_player {
                GameOutcome::Win
            } else {
                GameOutcome::Lose
            };
        }

        if self.field.iter().any(|x| x.is_none()) {
            GameOutcome::InProgress
        } else {
            GameOutcome::Draw
        }
    }

    fn get_available_moves(&self) -> Vec<Self::Move> {
        if self.outcome != GameOutcome::InProgress {
            return Vec::new();
        }

        self.field
            .iter()
            .enumerate()
            .filter(|(_, x)| x.is_none())
            .map(|(i, _)| i as u8)
            .collect()
    }

    fn perform_move(&mut self, b_move: &Self::Move) {
        self.field[*b_move as usize] = Some(self.current_player);
        self.current_player = match self.current_player {
            TTTPlayer::X => TTTPlayer::O,
            TTTPlayer::O => TTTPlayer::X,
        };
        self.outcome = self.get_outcome();
    }

    fn get_hash(&self) -> u128 {
        let mut hash = 0;
        for (i, &cell) in self.field.iter().enumerate() {
            let cell_value = match cell {
                None => 0,
                Some(TTTPlayer::X) => 1,
                Some(TTTPlayer::O) => 2,
            };
            hash += cell_value * 3u128.pow(i as u32);
        }
        hash
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum TTTPlayer {
    X,
    O,
}

#[cfg(test)]
mod tests {
    use crate::TicTacToeBoard;
    use mcts_lib::mcts::{DEFAULT_NODE_CAPACITY, MonteCarloTreeSearch};
    use mcts_lib::random::CustomNumberGenerator;

    #[test]
    fn test1_usual() {
        // arrange
        let board = TicTacToeBoard::default();
        let mut mcts = MonteCarloTreeSearch::builder(board)
            .with_alpha_beta_pruning(false)
            .with_random_generator(CustomNumberGenerator::default())
            .with_node_capacity(DEFAULT_NODE_CAPACITY)
            .build();

        // act
        mcts.iterate_n_times(20000);

        // assert
        let best_node = &mcts.get_most_perspective_move().data();
        assert_eq!(best_node.prev_move.unwrap(), 4);
        let root = &mcts.get_root().data();
        assert_eq!(root.wins, 13867);
        assert_eq!(root.draws, 2104);
        assert_eq!(root.visits, 20000);
        assert!(!root.is_fully_calculated);
    }

    #[test]
    fn test2_abp() {
        // arrange
        let board = TicTacToeBoard::default();
        let mut mcts = MonteCarloTreeSearch::builder(board)
            .with_random_generator(CustomNumberGenerator::default())
            .with_node_capacity(DEFAULT_NODE_CAPACITY)
            .build();

        // act
        mcts.iterate_n_times(20000);

        // assert
        let best_node = &mcts.get_most_perspective_move().data();
        assert_eq!(best_node.prev_move.unwrap(), 4);
        let root = &mcts.get_root().data();
        assert_eq!(root.wins, 10758);
        assert_eq!(root.draws, 3808);
        assert_eq!(root.visits, 20000);
        assert!(!root.is_fully_calculated);
    }

    #[test]
    fn test3_abp_fully_calculated() {
        // arrange
        let board = TicTacToeBoard::default();
        let mut mcts = MonteCarloTreeSearch::builder(board)
            .with_random_generator(CustomNumberGenerator::default())
            .with_node_capacity(DEFAULT_NODE_CAPACITY)
            .build();

        // act
        mcts.iterate_n_times(50000);

        // assert
        let best_node = &mcts.get_most_perspective_move().data();
        assert_eq!(best_node.prev_move.unwrap(), 4);
        let root = &mcts.get_root().data();
        assert_eq!(root.wins, 18225);
        assert_eq!(root.draws, 10342);
        assert_eq!(root.visits, 37432);
        assert!(root.is_fully_calculated);
    }
}
