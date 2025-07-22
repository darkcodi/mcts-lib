extern crate mcts_lib;

use mcts_lib::boards::tic_tac_toe::TicTacToeBoard;
use mcts_lib::mcts::MonteCarloTreeSearch;
use mcts_lib::random::CustomNumberGenerator;

fn main() {
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
    let root = mcts.get_root();
    for node in root.children() {
        println!(
            "Move: {:?} = {:.2?}%",
            node.value().prev_move,
            node.value().wins_rate() * 100.0
        );
    }

    // Get the most promising move
    let best_move_node = root.get_best_child().unwrap();
    let best_move = best_move_node.value().prev_move;

    println!("The best move is: {:?}", best_move);
    assert_eq!(best_move, Some(4));
}
