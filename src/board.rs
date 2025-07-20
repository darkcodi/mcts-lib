pub trait Board : Default + Clone {
    type Move;

    fn get_current_player(&self) -> Player;
    fn get_outcome(&self) -> GameOutcome;
    fn get_available_moves(&self) -> Vec<Self::Move>;
    fn perform_move(&mut self, b_move: &Self::Move);
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum GameOutcome {
    InProgress = 0,
    Win = 1,
    Lose = 2,
    Draw = 3,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Player {
    Me = 1,
    Other = 2,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Bound {
    None = 0,
    DefoWin = 1,
    DefoLose = 2,
}
