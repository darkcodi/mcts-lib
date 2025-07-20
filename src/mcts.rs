use crate::board::{Board, Bound, GameOutcome, Player};
use crate::hash::MurMurHasher;
use crate::mcts_node::MctsNode;
use crate::random::RandomGenerator;
use id_tree::InsertBehavior::{AsRoot, UnderNode};
use id_tree::{Node, NodeId, Tree, TreeBuilder};
use std::borrow::Borrow;

pub struct MonteCarloTreeSearch<T: Board, K: RandomGenerator> {
    tree: Tree<MctsNode<T>>,
    root_id: NodeId,
    random: K,
    use_alpha_beta_pruning: bool,
    next_action: MctsAction,
}

pub const DEFAULT_NODE_CAPACITY: usize = 10000;

impl<T: Board, K: RandomGenerator> Default for MonteCarloTreeSearch<T, K> {
    fn default() -> Self {
        MonteCarloTreeSearch::new(T::default(), K::default(), DEFAULT_NODE_CAPACITY, true)
    }
}

impl<T: Board, K: RandomGenerator> MonteCarloTreeSearch<T, K> {
    pub fn new(board: T, rg: K, node_capacity: usize, use_alpha_beta_pruning: bool) -> Self {
        let mut tree: Tree<MctsNode<T>> =
            TreeBuilder::new().with_node_capacity(node_capacity).build();
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

    pub fn get_tree(&self) -> &Tree<MctsNode<T>> {
        &self.tree
    }

    pub fn get_next_mcts_action(&self) -> &MctsAction {
        &self.next_action
    }

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

    pub fn iterate_n_times(&mut self, n: u32) {
        let mut iteration = 0;
        while iteration < n {
            self.do_iteration();
            iteration += 1;
        }
    }

    pub fn get_root(&self) -> &Node<MctsNode<T>> {
        let root = self.tree.get(self.root_id.borrow()).unwrap();
        root
    }

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

    pub fn get_tree_hash(&self) -> String {
        self.get_node_hash(&self.root_id)
    }

    pub fn get_node_hash(&self, node_id: &NodeId) -> String {
        let node = self.tree.get(node_id).unwrap();
        let outcome = match node.data().outcome {
            GameOutcome::InProgress => 0,
            GameOutcome::Win => 1,
            GameOutcome::Lose => 2,
            GameOutcome::Draw => 3,
        };
        let ifc = if node.data().is_fully_calculated {
            1
        } else {
            0
        };
        let mut str = format!(
            "[{}/{}/{}/{}/{}/{}/{};",
            node.data().id,
            node.data().height,
            node.data().wins,
            node.data().draws,
            node.data().visits,
            outcome,
            ifc
        );
        for child_id in node.children() {
            let child_hash = self.get_node_hash(child_id);
            str.push_str(child_hash.as_str());
        }
        str.push_str("]");
        MurMurHasher::hash(str.as_str())
    }

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

        let selected_child = self
            .random
            .get_random_from_vec(self.tree.get(node_id).unwrap().children())
            .clone();
        (new_node_ids, selected_child)
    }

    fn simulate(&mut self, node_id: &NodeId) -> GameOutcome {
        let node = self.tree.get(node_id).unwrap();
        let mut board = node.data().board.clone();
        let mut outcome = board.get_outcome();
        while outcome == GameOutcome::InProgress {
            let all_possible_moves = board.get_available_moves();
            let random_move = self.random.get_random_from_vec(&all_possible_moves);
            board.perform_move(random_move);
            outcome = board.get_outcome();
        }
        outcome
    }

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

/*
   Selection: Start from root R and select successive child nodes until a leaf node L is reached. The root is the current game state and a leaf is any node that has a potential child from which no simulation (playout) has yet been initiated. The section below says more about a way of biasing choice of child nodes that lets the game tree expand towards the most promising moves, which is the essence of Monte Carlo tree search.
   Expansion: Unless L ends the game decisively (e.g. win/loss/draw) for either player, create one (or more) child nodes and choose node C from one of them. Child nodes are any valid moves from the game position defined by L.
   Simulation: Complete one random playout from node C. This step is sometimes also called playout or rollout. A playout may be as simple as choosing uniform random moves until the game is decided (for example in chess, the game is won, lost, or drawn).
   Backpropagation: Use the result of the playout to update information in the nodes on the path from C to R.
*/
#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Clone)]
pub enum MctsAction {
    Selection {
        R: NodeId,       // Root
        RP: Vec<NodeId>, // RootPath
    },
    Expansion {
        L: NodeId,
    },
    Simulation {
        C: NodeId,       // Child
        AC: Vec<NodeId>, // AllChildren
    },
    Backpropagation {
        C: NodeId,
        result: GameOutcome,
    },
    EverythingIsCalculated,
}

impl MctsAction {
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
