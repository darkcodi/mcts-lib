use crate::board::{Board, Bound, GameOutcome, Player};
use crate::mcts_node::MctsNode;
use crate::random::{RandomGenerator, StandardRandomGenerator};
use id_tree::InsertBehavior::{AsRoot, UnderNode};
use id_tree::{Node, NodeId, Tree, TreeBuilder};
use std::borrow::Borrow;
use std::collections::HashSet;

/// The main struct for running the Monte Carlo Tree Search algorithm.
///
/// It holds the search tree, the random number generator, and the configuration for the search.
pub struct MonteCarloTreeSearch<T: Board, K: RandomGenerator> {
    tree: Tree<MctsNode<T>>,
    root_id: NodeId,
    random: K,
    use_alpha_beta_pruning: bool,
    next_action: MctsAction,
}

impl<T: Board, K: RandomGenerator> Default for MonteCarloTreeSearch<T, K> {
    fn default() -> Self {
        MonteCarloTreeSearchBuilder::new(T::default()).build()
    }
}

/// A builder for creating instances of `MonteCarloTreeSearch`.
///
/// This provides a convenient way to configure the MCTS search with different parameters.
pub struct MonteCarloTreeSearchBuilder<T: Board, K: RandomGenerator> {
    board: T,
    random_generator: K,
    use_alpha_beta_pruning: bool,
}

impl<T: Board, K: RandomGenerator> MonteCarloTreeSearchBuilder<T, K> {
    /// Creates a new builder with the given initial board state.
    pub fn new(board: T) -> Self {
        Self {
            board,
            random_generator: K::default(),
            use_alpha_beta_pruning: true,
        }
    }

    /// Sets the random number generator for the MCTS search.
    pub fn with_random_generator(mut self, rg: K) -> Self {
        self.random_generator = rg;
        self
    }

    /// Enables or disables alpha-beta pruning.
    pub fn with_alpha_beta_pruning(mut self, use_abp: bool) -> Self {
        self.use_alpha_beta_pruning = use_abp;
        self
    }

    /// Builds the `MonteCarloTreeSearch` instance with the configured parameters.
    pub fn build(self) -> MonteCarloTreeSearch<T, K> {
        MonteCarloTreeSearch::new(
            self.board,
            self.random_generator,
            self.use_alpha_beta_pruning,
        )
    }
}

impl<T: Board, K: RandomGenerator> MonteCarloTreeSearch<T, K> {
    /// Returns a new builder for `MonteCarloTreeSearch`.
    pub fn builder(board: T) -> MonteCarloTreeSearchBuilder<T, K> {
        MonteCarloTreeSearchBuilder::new(board)
    }

    /// Creates a new `MonteCarloTreeSearch` instance.
    ///
    /// It is recommended to use the builder pattern via `MonteCarloTreeSearch::builder()` instead.
    pub fn new(board: T, rg: K, use_alpha_beta_pruning: bool) -> Self {
        let mut tree: Tree<MctsNode<T>> = TreeBuilder::new().build();
        let root_mcts_node = MctsNode::new(0, Box::new(board));
        let root_id = tree.insert(Node::new(root_mcts_node), AsRoot).unwrap();

        Self {
            tree,
            root_id: root_id.clone(),
            random: rg,
            use_alpha_beta_pruning,
            next_action: MctsAction::Selection {
                R: root_id.clone(),
                RP: vec![],
            },
        }
    }

    /// Returns an immutable reference to the underlying search tree.
    pub fn get_tree(&self) -> &Tree<MctsNode<T>> {
        &self.tree
    }

    /// Returns the next MCTS action to be performed. Useful for debugging and visualization.
    pub fn get_next_mcts_action(&self) -> &MctsAction {
        &self.next_action
    }

    /// Executes a single step of the MCTS algorithm (Selection, Expansion, Simulation, or Backpropagation).
    pub fn execute_action(&mut self) {
        match self.next_action.clone() {
            MctsAction::Selection { R, RP: _cr } => {
                let maybe_selected_node = self.select_next_node(&R);
                self.next_action = match maybe_selected_node {
                    None => MctsAction::EverythingIsCalculated,
                    Some(selected_node) => MctsAction::Expansion { L: selected_node },
                };
            }
            MctsAction::Expansion { L } => {
                let (children, selected_child) = self.expand_node(&L);
                self.next_action = MctsAction::Simulation {
                    C: selected_child,
                    AC: children,
                };
            }
            MctsAction::Simulation { C, AC: _ac } => {
                let outcome = self.simulate(&C);
                self.next_action = MctsAction::Backpropagation { C, result: outcome };
            }
            MctsAction::Backpropagation { C, result } => {
                let affected_nodes = self.backpropagate(&C, result);
                self.next_action = MctsAction::Selection {
                    R: self.root_id.clone(),
                    RP: affected_nodes,
                }
            }
            MctsAction::EverythingIsCalculated => {}
        }
    }

    /// Performs one full iteration of the MCTS algorithm (Selection, Expansion, Simulation, Backpropagation).
    /// Returns the path of nodes that were updated during backpropagation.
    pub fn do_iteration(&mut self) -> Vec<NodeId> {
        self.execute_action();
        let mut is_selection = matches!(self.next_action, MctsAction::Selection { R: _, RP: _ });
        let mut is_fully_calculated =
            matches!(self.next_action, MctsAction::EverythingIsCalculated);
        while !is_selection && !is_fully_calculated {
            self.execute_action();
            is_selection = matches!(self.next_action, MctsAction::Selection { R: _, RP: _ });
            is_fully_calculated = matches!(self.next_action, MctsAction::EverythingIsCalculated);
        }

        match self.next_action.clone() {
            MctsAction::Selection { R: _, RP: rp } => rp,
            _ => vec![],
        }
    }

    /// Runs the MCTS search for a specified number of iterations.
    pub fn iterate_n_times(&mut self, n: u32) {
        let mut iteration = 0;
        while iteration < n {
            self.do_iteration();
            iteration += 1;
        }
    }

    /// Returns a reference to the root node of the search tree.
    pub fn get_root(&self) -> &Node<MctsNode<T>> {
        let root = self.tree.get(self.root_id.borrow()).unwrap();
        root
    }

    /// Returns the child of the root node that is considered the most promising, based on win rate.
    pub fn get_most_perspective_move(&self) -> &Node<MctsNode<T>> {
        let root = self.tree.get(self.root_id.borrow()).unwrap();
        let mut max_win_rate = 0.0;
        let mut best_node = root;
        for node_id in root.children() {
            let node = self.tree.get(node_id).unwrap();
            let node_win_rate = node.data().wins_rate();
            if node_win_rate > max_win_rate {
                max_win_rate = node_win_rate;
                best_node = node;
            }
        }

        best_node
    }

    /// Selects the most promising node to expand, using the UCB1 formula.
    fn select_next_node(&self, root_id: &NodeId) -> Option<NodeId> {
        let mut promising_node_id = &root_id.clone();
        let mut has_changed = false;
        loop {
            let mut best_child_id: Option<&NodeId> = None;
            let mut max_ucb = f64::MIN;
            let node = self.tree.get(promising_node_id).unwrap();
            for child_id in node.children() {
                let child = self.tree.get(child_id).unwrap();
                if child.data().is_fully_calculated {
                    continue;
                }

                let current_ucb = MonteCarloTreeSearch::<T, K>::ucb_value(
                    node.data().visits,
                    child.data().wins,
                    child.data().visits,
                );
                if current_ucb > max_ucb {
                    max_ucb = current_ucb;
                    best_child_id = Some(child_id);
                }
            }
            if best_child_id.is_none() {
                break;
            }
            promising_node_id = best_child_id.unwrap();
            has_changed = true;
        }

        if has_changed {
            Some(promising_node_id.clone())
        } else {
            let root = self.tree.get(&root_id.clone()).unwrap();
            if root.children().is_empty() {
                Some(root_id.clone())
            } else {
                None
            }
        }
    }

    /// Expands a leaf node by creating its children, representing all possible moves from that state.
    fn expand_node(&mut self, node_id: &NodeId) -> (Vec<NodeId>, NodeId) {
        let node = self.tree.get(node_id).unwrap();
        if !node.children().is_empty() {
            panic!("BUG: expanding already expanded node");
        }
        if node.data().outcome != GameOutcome::InProgress {
            return (vec![], node_id.clone());
        }

        let children_height = node.data().height + 1;
        let all_possible_moves = node.data().board.get_available_moves();
        let mut new_mcts_nodes = Vec::with_capacity(all_possible_moves.len());

        for possible_move in all_possible_moves {
            let mut board_clone = node.data().board.clone();
            board_clone.perform_move(&possible_move);
            let new_node_id = self.random.next();
            let mut mcts_node = MctsNode::new(new_node_id, board_clone);
            mcts_node.prev_move = Some(possible_move);
            mcts_node.height = children_height;
            new_mcts_nodes.push(mcts_node);
        }

        let mut new_node_ids = Vec::with_capacity(new_mcts_nodes.len());
        for mcts_node in new_mcts_nodes {
            let node_id = self
                .tree
                .insert(Node::new(mcts_node), UnderNode(node_id))
                .unwrap();
            new_node_ids.push(node_id);
        }

        let children = self.tree.get(node_id).unwrap().children();
        let selected_child_index = self.random.next_range(0, children.len() as i32) as usize;
        let selected_child = children[selected_child_index].clone();
        (new_node_ids, selected_child)
    }

    /// Simulates a random playout from a given node until the game ends.
    fn simulate(&mut self, node_id: &NodeId) -> GameOutcome {
        let node = self.tree.get(node_id).unwrap();
        let mut board = node.data().board.clone();
        let mut outcome = board.get_outcome();
        let mut visited_states = HashSet::new();
        visited_states.insert(board.get_hash());

        while outcome == GameOutcome::InProgress {
            let mut all_possible_moves = board.get_available_moves();

            while !all_possible_moves.is_empty() {
                let random_move_index =
                    self.random.next_range(0, all_possible_moves.len() as i32) as usize;
                let random_move = all_possible_moves.get(random_move_index).unwrap();
                let mut new_board = board.clone();
                new_board.perform_move(random_move);
                let new_board_hash = new_board.get_hash();
                if visited_states.contains(&new_board_hash) {
                    all_possible_moves.remove(random_move_index);
                    continue;
                } else {
                    visited_states.insert(new_board_hash);
                    board = new_board;
                    break;
                }
            }

            if all_possible_moves.is_empty() {
                return GameOutcome::Draw;
            }

            outcome = board.get_outcome();
        }
        outcome
    }

    /// Propagates the result of a simulation back up the tree, updating node statistics.
    fn backpropagate(&mut self, node_id: &NodeId, outcome: GameOutcome) -> Vec<NodeId> {
        let mut branch = vec![node_id.clone()];

        loop {
            let temp_node = self.tree.get(branch.last().unwrap()).unwrap();
            let parent = temp_node.parent();
            if parent.is_none() {
                break;
            }

            branch.push(parent.unwrap().clone());
        }

        let is_win = outcome == GameOutcome::Win;
        let is_draw = outcome == GameOutcome::Draw;

        for node_id in &branch {
            let bound = self.get_bound(node_id);
            let is_fully_calculated = self.is_fully_calculated(node_id, bound);
            let temp_node = self.tree.get_mut(node_id).unwrap();
            let mcts_node = temp_node.data_mut();
            mcts_node.visits += 1;
            if is_win {
                mcts_node.wins += 1;
            }

            if is_draw {
                mcts_node.draws += 1;
            }

            if is_fully_calculated {
                mcts_node.is_fully_calculated = true;
            }

            if bound != Bound::None {
                mcts_node.bound = bound;
            }
        }

        branch
    }

    /// Determines the bound of a node for alpha-beta pruning.
    fn get_bound(&self, node_id: &NodeId) -> Bound {
        if !self.use_alpha_beta_pruning {
            return Bound::None;
        }

        let node = self.tree.get(node_id).unwrap();
        let mcts_node = node.data();
        if mcts_node.bound != Bound::None {
            return mcts_node.bound;
        }

        if mcts_node.outcome == GameOutcome::Win {
            return Bound::DefoWin;
        }

        if mcts_node.outcome == GameOutcome::Lose {
            return Bound::DefoLose;
        }

        if node.children().is_empty() {
            return Bound::None;
        }

        match mcts_node.current_player {
            Player::Me => {
                if node
                    .children()
                    .iter()
                    .all(|x| self.tree.get(x).unwrap().data().bound == Bound::DefoLose)
                {
                    return Bound::DefoLose;
                }

                if node
                    .children()
                    .iter()
                    .any(|x| self.tree.get(x).unwrap().data().bound == Bound::DefoWin)
                {
                    return Bound::DefoWin;
                }
            }
            Player::Other => {
                if node
                    .children()
                    .iter()
                    .all(|x| self.tree.get(x).unwrap().data().bound == Bound::DefoWin)
                {
                    return Bound::DefoWin;
                }

                if node
                    .children()
                    .iter()
                    .any(|x| self.tree.get(x).unwrap().data().bound == Bound::DefoLose)
                {
                    return Bound::DefoLose;
                }
            }
        }

        Bound::None
    }

    /// Checks if a node can be considered fully calculated, meaning its outcome is certain.
    fn is_fully_calculated(&self, node_id: &NodeId, bound: Bound) -> bool {
        if bound != Bound::None {
            return true;
        }

        let node = self.tree.get(node_id).unwrap();
        if node.data().outcome != GameOutcome::InProgress {
            return true;
        }

        if node.children().is_empty() {
            return false;
        }

        let all_children_calculated = node
            .children()
            .iter()
            .all(|x| self.tree.get(x).unwrap().data().is_fully_calculated);

        all_children_calculated
    }

    /// Calculates the UCB1 (Upper Confidence Bound 1) value for a node.
    fn ucb_value(total_visits: i32, node_wins: i32, node_visit: i32) -> f64 {
        const EXPLORATION_PARAMETER: f64 = std::f64::consts::SQRT_2;

        if node_visit == 0 {
            i32::MAX.into()
        } else {
            ((node_wins as f64) / (node_visit as f64))
                + EXPLORATION_PARAMETER
                    * f64::sqrt(f64::ln(total_visits as f64) / (node_visit as f64))
        }
    }
}

impl<T: Board> MonteCarloTreeSearch<T, StandardRandomGenerator> {
    pub fn from_board(board: T) -> Self {
        MonteCarloTreeSearchBuilder::new(board).build()
    }
}

/// Represents the four main stages of the MCTS algorithm.
///
/// This enum is used to manage the state of the search process.
#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Clone)]
pub enum MctsAction {
    /// **Selection**: Start from the root `R` and select successive child nodes until a leaf node `L` is reached.
    Selection {
        /// The root of the current selection phase.
        R: NodeId,
        /// The path of nodes visited during the last backpropagation phase.
        RP: Vec<NodeId>,
    },
    /// **Expansion**: Create one or more child nodes from the selected leaf node `L`.
    Expansion {
        /// The leaf node to be expanded.
        L: NodeId,
    },
    /// **Simulation**: Run a random playout from a newly created child node `C`.
    Simulation {
        /// The child node from which the simulation will start.
        C: NodeId,
        /// All children created during the expansion phase.
        AC: Vec<NodeId>,
    },
    /// **Backpropagation**: Update the statistics of the nodes on the path from `C` to the root `R`.
    Backpropagation {
        /// The child node from which the simulation was run.
        C: NodeId,
        /// The result of the simulation.
        result: GameOutcome,
    },
    /// Represents a state where the entire tree has been explored and the outcome is certain.
    EverythingIsCalculated,
}

impl MctsAction {
    /// Returns the name of the current MCTS action as a string.
    pub fn get_name(&self) -> String {
        match self {
            MctsAction::Selection { R: _, RP: _ } => "Selection".to_string(),
            MctsAction::Expansion { L: _ } => "Expansion".to_string(),
            MctsAction::Simulation { C: _, AC: _ } => "Simulation".to_string(),
            MctsAction::Backpropagation { C: _, result: _ } => "Backpropagation".to_string(),
            MctsAction::EverythingIsCalculated => "EverythingIsCalculated".to_string(),
        }
    }
}
