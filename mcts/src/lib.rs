extern crate rand;

use rand::distributions::{IndependentSample, Range};
use std::cmp::Ordering;
use std::fmt;
use std::mem;
use std::time;
use rand::Rng;

#[derive(Debug, PartialEq)]
pub struct Node<S: State> {
    action: Option<S::Action>,
    visits: usize,
    value: f64,
    untried_actions: S::Actions,
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
        match self.untried_actions.next() {
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
                let outcome = state.do_action(action);
                self.children.push(Node::new(
                    Some(action),
                    self.just_acted.other(),
                    state,
                    outcome,
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
        outcome: Outcome<S::Actions>,
        perspective: Player,
        rng: &mut R,
    ) -> Node<S> {
        let value = state.playout(rng, perspective, outcome.clone());
        Node {
            action,
            visits: 1,
            value,
            untried_actions: outcome.as_actions(),
            children: Vec::new(),
            just_acted,
        }
    }
    pub fn shallow_str(&self) -> String {
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
    pub fn print_1_layer(&self) {
        println!("{}", self.shallow_str());
        for ref child in self.children.iter() {
            println!("  {}", child.shallow_str());
        }
    }
    pub fn min_depth(&self) -> usize {
        self.children
            .iter()
            .map(|c| c.min_depth() + 1)
            .min()
            .unwrap_or(0)
    }
    pub fn visits(&self) -> usize {
        self.visits
    }
    pub fn value(&self) -> f64 {
        self.value
    }
    pub fn max_depth(&self) -> usize {
        self.children
            .iter()
            .map(|c| c.min_depth() + 1)
            .max()
            .unwrap_or(0)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Player {
    P1,
    P2,
}

impl Player {
    pub fn other(self) -> Self {
        match self {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        }
    }
}

#[derive(Clone)]
pub enum Outcome<Actions: Clone> {
    P1Win,
    P2Win,
    Draw,
    Actions(Actions),
}

impl<Actions: Default + Clone> Outcome<Actions> {
    fn value(&self, player: Player) -> f64 {
        match (self, player) {
            (&Outcome::P1Win, Player::P1) => 1.0,
            (&Outcome::P1Win, Player::P2) => 0.0,
            (&Outcome::P2Win, Player::P1) => 0.0,
            (&Outcome::P2Win, Player::P2) => 1.0,
            _ => 0.5,
        }
    }
    pub fn from_player(player: Player) -> Self {
        match player {
            Player::P1 => Outcome::P1Win,
            Player::P2 => Outcome::P2Win,
        }
    }
    fn as_actions(self) -> Actions {
        match self {
            Outcome::Actions(actions) => actions,
            _ => Actions::default(),
        }
    }
}

pub trait State: Clone + fmt::Display {
    type Action: Copy + Eq + fmt::Debug;
    type Actions: ExactSizeIterator + Iterator<Item=Self::Action> + Clone + Default + fmt::Debug;
    fn initial() -> Self;
    fn do_action(&mut self, action: Self::Action) -> Outcome<Self::Actions>;
    fn next_player(&self) -> Player;
    fn valid_actions(&self, player: Player) -> Self::Actions;
    fn has_won(&self, player: Player) -> bool;
    fn outcome(&self) -> Outcome<Self::Actions> {
        return if self.has_won(Player::P1) {
            Outcome::P1Win
        } else if self.has_won(Player::P2) {
            Outcome::P2Win
        } else {
            let actions = self.valid_actions(self.next_player());
            if actions.len() == 0 { Outcome::Draw } else { Outcome::Actions(actions) }
        }
    }
    fn playout<R: Rng>(&mut self, rng: &mut R, player: Player, mut outcome: Outcome<Self::Actions>) -> f64 {
        loop {
            let mut actions = if let Outcome::Actions(a) = outcome {
                a
            } else {
                return outcome.value(player);
            };
            let range = Range::new(0, actions.len());
            let action = actions.nth(range.ind_sample(rng)).unwrap();
            outcome = self.do_action(action);
        }
    }
}

pub struct MCTree<S: State, R: Rng> {
    pub root: Node<S>,
    state: S,
    rng: R,
    perspective: Player,
}

impl<S: State> MCTree<S, rand::ThreadRng> {
    pub fn search_for(&mut self, milliseconds: usize) {
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
    pub fn choose_and_do_action(&mut self) -> S::Action {
        assert!(self.perspective != self.root.just_acted);
        let action = self.root.best_action().unwrap();
        self.do_action(action);
        action
    }
    pub fn do_action(&mut self, action: S::Action) {
        let index = self.root
            .children
            .iter()
            .position(|c| c.action == Some(action))
            .unwrap();
        let new_root = self.root.children.remove(index);
        let old_root = mem::replace(&mut self.root, new_root);
        old_root.action.map(|a| self.state.do_action(a));
    }
    pub fn new(state: S, perspective: Player, to_move: Player) -> Self {
        let mut rng = rand::thread_rng();
        MCTree {
            root: Node::new(None, to_move.other(), state.clone(), state.outcome(), perspective, &mut rng),
            state,
            rng,
            perspective,
        }
    }
}
