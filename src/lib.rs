//! A small and simple library for Monte Carlo tree search.
//!
//! This library provides a generic implementation of the Monte Carlo Tree Search (MCTS) algorithm.
//! MCTS is a heuristic search algorithm used in decision-making processes, most notably in game AI.
//! The library is designed to be flexible and adaptable to various turn-based games.

/// Contains the `Board` trait and related enums that define the interface for a game.
pub mod board;
/// The core module of the library, containing the `MonteCarloTreeSearch` implementation.
pub mod mcts;
/// Contains the `MctsNode` struct, which represents a node in the search tree.
pub mod mcts_node;
/// Contains traits and implementations for random number generation.
pub mod random;
