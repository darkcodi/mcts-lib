/// The central trait of the library, defining the interface for a game state.
///
/// To use the MCTS algorithm with a custom game, this trait must be implemented.
/// It provides the MCTS engine with the necessary methods to understand and interact with the game logic.
pub trait Board: Default + Clone {
    /// The type representing a move in the game. This could be a simple `u8` for a board position
    /// or a more complex struct for games with intricate actions.
    type Move;

    /// Returns the player whose turn it is to make a move.
    fn get_current_player(&self) -> Player;

    /// Returns the current outcome of the game.
    fn get_outcome(&self) -> GameOutcome;

    /// Returns a list of all legal moves available from the current state.
    fn get_available_moves(&self) -> Vec<Self::Move>;

    /// Applies a given move to the board, modifying its state.
    fn perform_move(&mut self, b_move: &Self::Move);

    /// Returns a hash value for the current board state.
    fn get_hash(&self) -> u128;
}

/// Represents the possible outcomes of a game.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum GameOutcome {
    /// The game is still ongoing.
    InProgress = 0,
    /// The current player has won.
    Win = 1,
    /// The current player has lost.
    Lose = 2,
    /// The game has ended in a draw.
    Draw = 3,
}

/// Represents the players in the game from the perspective of the MCTS search.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Player {
    /// The player for whom the MCTS is currently searching for the best move.
    Me = 1,
    /// The opponent.
    Other = 2,
}

/// Used for alpha-beta pruning to mark nodes as having a definite outcome.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Bound {
    /// The outcome of the node is not yet determined.
    None = 0,
    /// This node is a guaranteed win for the current player.
    DefoWin = 1,
    /// This node is a guaranteed loss for the current player.
    DefoLose = 2,
}
