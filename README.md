# mcts-lib

A small and simple library for Monte Carlo tree search.

[![crates.io](https://img.shields.io/crates/v/mcts-lib.svg)](https://crates.io/crates/mcts-lib)
[![docs.rs](https://docs.rs/mcts-lib/badge.svg)](https://docs.rs/mcts-lib)

This library provides a generic implementation of the Monte Carlo Tree Search (MCTS) algorithm in Rust. MCTS is a powerful heuristic search algorithm for decision-making processes, particularly in games. This library is designed to be flexible and easy to integrate with various turn-based games.

## Features

-   Generic implementation of the MCTS algorithm.
-   Flexible `Board` trait for easy integration with your own games.
-   Includes an example implementation for Tic-Tac-Toe.
-   Alpha-beta pruning for optimization.

## Getting Started

## Usage

To use this library, you need to implement the `Board` trait for your game's state representation. Here's a high-level overview of the steps:

1.  **Define your game state:** Create a struct or enum to represent your game's state.
2.  **Implement the `Board` trait:** Implement the `Board` trait for your game state. This involves defining the logic for:
    *   Getting the current player.
    *   Determining the game's outcome (win, lose, draw, in-progress).
    *   Listing available moves.
    *   Applying a move to the board.
3.  **Configure `MonteCarloTreeSearch`:** Use the `MonteCarloTreeSearch::builder()` to create and configure an instance of the search algorithm.
4.  **Run the search:** Use `iterate_n_times` to run the MCTS algorithm.
5.  **Get the best move:** Use `get_most_perspective_move` to get the best move found by the algorithm.

### Example: Tic-Tac-Toe

The library includes a Tic-Tac-Toe implementation that you can use as a reference. See `examples/tic_tac_toe.rs`.

```rust
use mcts_lib::boards::tic_tac_toe::TicTacToeBoard;
use mcts_lib::mcts::MonteCarloTreeSearch;
use mcts_lib::random::CustomNumberGenerator;

// Create a new Tic-Tac-Toe board
let board = TicTacToeBoard::default();

// Create a new MCTS search instance
let mut mcts = MonteCarloTreeSearch::builder(board)
    .with_alpha_beta_pruning(false)
    .with_random_generator(CustomNumberGenerator::default())
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
```

## Building and Testing

-   **Build:** `cargo build`
-   **Test:** `cargo test`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
