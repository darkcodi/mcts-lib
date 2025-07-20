use crate::board::{Board, Bound, GameOutcome, Player};

pub struct MctsNode<T: Board> {
    pub id: i32,
    pub height: i32,
    pub board: Box<T>,
    pub prev_move: Option<T::Move>,
    pub current_player: Player,
    pub outcome: GameOutcome,
    pub visits: i32,
    pub wins: i32,
    pub draws: i32,
    pub bound: Bound,
    pub is_fully_calculated: bool,
}

impl<T: Board> Default for MctsNode<T> {
    fn default() -> Self {
        MctsNode::new(0, Box::new(T::default()))
    }
}

impl<T: Board> MctsNode<T> {
    pub fn new(id: i32, boxed_board: Box<T>) -> Self {
        let player = boxed_board.get_current_player();
        let outcome = boxed_board.get_outcome();
        MctsNode {
            id,
            height: 0,
            board: boxed_board,
            prev_move: None,
            current_player: player,
            outcome: outcome,
            visits: 0,
            wins: 0,
            draws: 0,
            bound: Bound::None,
            is_fully_calculated: false,
        }
    }

    pub fn wins_rate(&self) -> f64 {
        (self.wins as f64) / (self.visits as f64)
    }

    pub fn draws_rate(&self) -> f64 {
        (self.draws as f64) / (self.visits as f64)
    }
}