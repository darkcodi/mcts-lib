use crate::board::{Board, Bound, GameOutcome, Player};

/// Represents a single node in the Monte Carlo search tree.
///
/// Each node stores the state of the game, statistics about the outcomes of simulations,
/// and information about the move that led to this state.
#[derive(Debug, Clone)]
pub struct MctsNode<T: Board> {
    /// A unique identifier for the node.
    pub id: i32,
    /// The depth of the node in the tree.
    pub height: i32,
    /// The game state that this node represents.
    pub board: Box<T>,
    /// The move that led to this node's state from its parent. `None` for the root node.
    pub prev_move: Option<T::Move>,
    /// The player whose turn it is in this node's game state.
    pub current_player: Player,
    /// The outcome of the game at this node, if it is terminal.
    pub outcome: GameOutcome,
    /// The number of times this node has been visited during the search.
    pub visits: i32,
    /// The number of times simulations from this node have resulted in a win for the current player.
    pub wins: i32,
    /// The number of times simulations from this node have resulted in a draw.
    pub draws: i32,
    /// The bound of the node, used for alpha-beta pruning.
    pub bound: Bound,
    /// A flag indicating whether the outcome of this node is definitively known.
    pub is_fully_calculated: bool,
}

impl<T: Board> Default for MctsNode<T> {
    fn default() -> Self {
        MctsNode::new(0, Box::new(T::default()))
    }
}

impl<T: Board> MctsNode<T> {
    /// Creates a new `MctsNode` with the given ID and board state.
    pub fn new(id: i32, boxed_board: Box<T>) -> Self {
        let player = boxed_board.get_current_player();
        let outcome = boxed_board.get_outcome();
        MctsNode {
            id,
            height: 0,
            board: boxed_board,
            prev_move: None,
            current_player: player,
            outcome,
            visits: 0,
            wins: 0,
            draws: 0,
            bound: Bound::None,
            is_fully_calculated: false,
        }
    }

    /// Calculates the win rate of this node.
    pub fn wins_rate(&self) -> f64 {
        if self.visits == 0 {
            0.0
        } else {
            (self.wins as f64) / (self.visits as f64)
        }
    }

    /// Calculates the draw rate of this node.
    pub fn draws_rate(&self) -> f64 {
        if self.visits == 0 {
            0.0
        } else {
            (self.draws as f64) / (self.visits as f64)
        }
    }
}

impl<T: Board> PartialEq<Self> for MctsNode<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: Board> Eq for MctsNode<T> {}

impl<T: Board> std::hash::Hash for MctsNode<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
