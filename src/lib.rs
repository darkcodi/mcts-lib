//! A small and simple library for Monte Carlo tree search.
//!
//! This library provides a generic implementation of the Monte Carlo Tree Search (MCTS) algorithm.
//! MCTS is a heuristic search algorithm used in decision-making processes, most notably in game AI.
//! The library is designed to be flexible and adaptable to various turn-based games.
//!
//! # Example
//!
//! ```rust
//! use mcts_lib::boards::tic_tac_toe::TicTacToeBoard;
//! use mcts_lib::mcts::{MonteCarloTreeSearch, DEFAULT_NODE_CAPACITY};
//! use mcts_lib::random::CustomNumberGenerator;
//!
//! // Create a new Tic-Tac-Toe board
//! let board = TicTacToeBoard::default();
//!
//! // Create and configure a new MCTS search instance using the builder
//! let mut mcts = MonteCarloTreeSearch::builder(board)
//!     .with_random_generator(CustomNumberGenerator::default())
//!     .with_node_capacity(DEFAULT_NODE_CAPACITY)
//!     .with_alpha_beta_pruning(true)
//!     .build();
//!
//! // Run the search for a number of iterations
//! mcts.iterate_n_times(1000);
//!
//! // Get the most promising move
//! let best_move_node = mcts.get_most_perspective_move();
//! let best_move = best_move_node.data().prev_move;
//!
//! println!("The best move is: {:?}", best_move);
//! ```

/// Contains the `Board` trait and related enums that define the interface for a game.
pub mod board;
/// Contains pre-made implementations of the `Board` trait for common games.
pub mod boards;
/// The core module of the library, containing the `MonteCarloTreeSearch` implementation.
pub mod mcts;
/// Contains the `MctsNode` struct, which represents a node in the search tree.
pub mod mcts_node;
/// Contains traits and implementations for random number generation.
pub mod random;
