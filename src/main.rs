extern crate rand;

use rand::distributions::{IndependentSample, Range};
use std::cmp::Ordering;
use std::fmt;
use std::io;
use std::mem;
use std::time;
use std::env;
use rand::Rng;

use std::str::FromStr;

#[derive(Debug, PartialEq)]
struct Node<S: State> {
    action: Option<S::Action>,
    visits: usize,
    value: f64,
    untried_actions: Vec<S::Action>,
    children: Vec<Node<S>>,
    just_acted: Player,
}

fn f64_cmp(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Less)
}

impl<S: State> Node<S> {
    /// Returns the value of the result
    fn select<R: Rng>(&mut self, mut state: S, rng: &mut R, player: Player) -> f64 {
        self.action.map(|a| state.do_action(a));
        match self.untried_actions.pop() {
            None => {
                if self.children.len() == 0 {
                    self.visits += 1;
                    self.value
                } else {
                    let max = player != self.just_acted;
                    let val = self.choose_child(max).unwrap().select(state, rng, player);
                    self.value = (self.value * self.visits as f64 + val) /
                        (self.visits as f64 + 1.0);
                    self.visits += 1;
                    val
                }
            }
            Some(action) => {
                state.do_action(action);
                self.children.push(Node::new(
                    Some(action),
                    self.just_acted.other(),
                    state,
                    player,
                    rng,
                ));
                let val = self.children.last().unwrap().value;
                self.value = (self.value * self.visits as f64 + val) / (self.visits as f64 + 1.0);
                self.visits += 1;
                val
            }
        }
    }
    fn choose_child(&mut self, max: bool) -> Option<&mut Node<S>> {
        let visits: usize = self.visits;
        let weight = |c: &Node<S>| if max { c.value } else { 1.0 - c.value } +
            ((visits as f64 * 2.0).ln() / c.visits as f64).sqrt();
        self.children.iter_mut().max_by(
            |a, b| f64_cmp(weight(a), weight(b)),
        )
    }
    fn best_action(&self) -> Option<S::Action> {
        self.children
            .iter()
            .max_by(|a, b| f64_cmp(a.value, b.value))
            .and_then(|c| c.action)
    }
    fn new<R: Rng>(
        action: Option<S::Action>,
        just_acted: Player,
        mut state: S,
        perspective: Player,
        rng: &mut R,
    ) -> Node<S> {
        let actions = state.valid_actions(just_acted.other());
        let value = state.playout(rng, perspective);
        Node {
            action,
            visits: 1,
            value,
            untried_actions: actions,
            children: Vec::new(),
            just_acted,
        }
    }
    fn shallow_str(&self) -> String {
        format!(
            "Node ( Just = {:?}{:?}, value = {}, visits = {}, untried = {:?}, chidren: {} )",
            self.just_acted,
            self.action,
            self.value,
            self.visits,
            self.untried_actions,
            self.children.len()
        )
    }
    #[allow(dead_code)]
    fn print_1_layer(&self) {
        println!("{}", self.shallow_str());
        for ref child in self.children.iter() {
            println!("  {}", child.shallow_str());
        }
    }
    fn min_depth(&self) -> usize {
        self.children
            .iter()
            .map(|c| c.min_depth() + 1)
            .min()
            .unwrap_or(0)
    }
    fn max_depth(&self) -> usize {
        self.children
            .iter()
            .map(|c| c.min_depth() + 1)
            .max()
            .unwrap_or(0)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Player {
    P1,
    P2,
}

impl Player {
    fn other(self) -> Self {
        match self {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        }
    }
}

trait State: Clone {
    type Action: Copy + Eq + fmt::Debug;
    fn initial() -> Self;
    fn do_action(&mut self, action: Self::Action);
    fn next_player(&self) -> Player;
    fn valid_actions(&self, player: Player) -> Vec<Self::Action>;
    fn value(&self, player: Player) -> f64 {
        if self.has_won(player) {
            1.0
        } else if self.has_won(player.other()) {
            0.0
        } else {
            0.5
        }
    }
    fn has_won(&self, player: Player) -> bool;
    fn playout<R: Rng>(&mut self, rng: &mut R, player: Player) -> f64 {
        loop {
            let actions = self.valid_actions(self.next_player());
            if actions.len() == 0 {
                return self.value(player);
            }
            let range = Range::new(0, actions.len());
            self.do_action(actions[range.ind_sample(rng)])
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum C4Cell {
    O,
    X,
    Blank,
}

impl fmt::Display for C4Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                C4Cell::O => "O",
                C4Cell::X => "X",
                C4Cell::Blank => " ",
            }
        )
    }
}

#[derive(Clone)]
struct C4State {
    board: [C4Cell; 42],
    next: Player,
}

impl fmt::Display for C4State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for r in 0..6 {
            write!(f, "|")?;
            for c in 0..7 {
                if c != 0 {
                    write!(f, " ")?;
                }
                write!(f, "{}", self.get(r, c))?;
            }
            writeln!(f, "|")?;
        }
        writeln!(f, "+-------------+")?;
        writeln!(f, "|0 1 2 3 4 5 6|")?;
        write!(f, "+-------------+")
    }
}

impl C4State {
    fn get(&self, row: u8, col: u8) -> C4Cell {
        self.board[(row * 7 + col) as usize]
    }
    fn piece(p: Player) -> C4Cell {
        match p {
            Player::P1 => C4Cell::X,
            Player::P2 => C4Cell::O,
        }
    }
    fn play(&mut self, row: u8, col: u8, player: Player) {
        self.board[(row * 7 + col) as usize] = C4State::piece(player);
    }
}

impl State for C4State {
    type Action = u8;

    fn initial() -> Self {
        C4State {
            board: [C4Cell::Blank; 42],
            next: Player::P1,
        }
    }

    fn next_player(&self) -> Player {
        self.next
    }

    fn do_action(&mut self, col: Self::Action) {
        for row in (0..6).rev() {
            if self.get(row, col) == C4Cell::Blank {
                let player = self.next;
                self.play(row, col, player);
                self.next = self.next.other();
                break;
            }
        }
    }

    fn valid_actions(&self, _: Player) -> Vec<Self::Action> {
        if !self.has_won(Player::P1) && !self.has_won(Player::P2) {
            (0..7)
                .filter(|col| self.get(0, *col) == C4Cell::Blank)
                .collect()
        } else {
            Vec::new()
        }
    }

    fn has_won(&self, player: Player) -> bool {
        let piece = C4State::piece(player);
        let streak = 4;
        let rows = 6;
        let cols = 7;

        // Column wins
        for c in 0..(cols) {
            for r in 0..(rows - streak + 1) {
                if (0..streak).all(|i| self.get(r + i, c) == piece) {
                    return true;
                }
            }
        }

        // Check row wins
        for r in 0..(rows) {
            for c in 0..(cols - streak + 1) {
                if (0..streak).all(|i| self.get(r, c + i) == piece) {
                    return true;
                }
            }
        }

        // Check for back-diagonal wins
        for r in 0..(rows - streak + 1) {
            for c in 0..(cols - streak + 1) {
                if (0..streak).all(|i| self.get(r + i, c + i) == piece) {
                    return true;
                }
            }
        }

        // Check for forward-diagonal wins
        for r in (streak - 1)..rows {
            for c in 0..(cols - streak + 1) {
                if (0..streak).all(|i| self.get(r - i, c + i) == piece) {
                    return true;
                }
            }
        }

        false
    }
}

struct MCTree<S: State, R: Rng> {
    root: Node<S>,
    state: S,
    rng: R,
    perspective: Player,
}

impl<S: State> MCTree<S, rand::ThreadRng> {
    fn search_for(&mut self, milliseconds: usize) {
        let start = time::Instant::now();
        let duration = time::Duration::from_millis(milliseconds as u64);
        let mut searches = 0;
        while start.elapsed() < duration {
            searches += 1;
            self.iter();
        }
        println!("Did {} searches in {} milliseconds", searches, milliseconds);
    }
    fn iter(&mut self) {
        self.root.select(
            self.state.clone(),
            &mut self.rng,
            self.perspective,
        );
    }
    fn choose_and_do_action(&mut self) -> S::Action {
        assert!(self.perspective != self.root.just_acted);
        let action = self.root.best_action().unwrap();
        self.do_action(action);
        action
    }
    fn do_action(&mut self, action: S::Action) {
        let index = self.root
            .children
            .iter()
            .position(|c| c.action == Some(action))
            .unwrap();
        let new_root = self.root.children.remove(index);
        let old_root = mem::replace(&mut self.root, new_root);
        old_root.action.map(|a| self.state.do_action(a));
    }
    fn new(state: S, perspective: Player, to_move: Player) -> Self {
        let mut rng = rand::thread_rng();
        MCTree {
            root: Node::new(None, to_move.other(), state.clone(), perspective, &mut rng),
            state,
            rng,
            perspective,
        }
    }
}

fn get_column(s: &C4State) -> u8 {
    let mut line = String::new();
    loop {
        println!("Enter a column: ");
        io::stdin().read_line(&mut line).unwrap();
        let col = match line.as_str().trim() {
            "0" => 0,
            "1" => 1,
            "2" => 2,
            "3" => 3,
            "4" => 4,
            "5" => 5,
            "6" => 6,
            _ => 7,
        };
        if col < 7 && s.get(0, col) == C4Cell::Blank {
            return col;
        }
        println!("Invalid column!");
        line.clear();
    }
}

#[allow(dead_code)]
fn mcts(thinking_time: usize) {
    let mut board = C4State::initial();
    let mut mctree = MCTree::new(board.clone(), Player::P2, Player::P1);
    mctree.search_for(thinking_time);
    println!("{}", board);
    loop {
        let user_col = get_column(&board);
        board.do_action(user_col);
        if board.has_won(Player::P1) {
            println!("X Won!");
            break;
        }
        println!("{}", board);
        mctree.do_action(user_col);
        mctree.search_for(thinking_time);
        let ai_col = mctree.choose_and_do_action();
        board.do_action(ai_col);
        println!("The AI played column {}", ai_col);
        println!(
            " it has played {} games from this position",
            mctree.root.visits
        );
        println!(
            " and it believes it will win with p = {}",
            mctree.root.value
        );
        println!(
            " it has explored {} moves ahead fully, and has ventured as far as {} moves",
            mctree.root.min_depth(),
            mctree.root.max_depth()
        );
        println!("{}", board);
        if board.has_won(Player::P2) {
            println!("O Won!");
            break;
        }
        if board.valid_actions(Player::P1).len() == 0 {
            println!("Draw");
            break;
        }
    }
}

fn main() {
    let thinking_time = env::args().nth(1).and_then(|a| usize::from_str(&a).ok()).unwrap_or(3000);
    mcts(thinking_time)
}
