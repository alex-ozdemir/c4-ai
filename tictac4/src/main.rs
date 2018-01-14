extern crate mcts;

use std::fmt;
use std::io;
use std::env;
use mcts::*;

use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum T4Cell {
    O,
    X,
    Blank,
}

impl fmt::Display for T4Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                T4Cell::O => "O",
                T4Cell::X => "X",
                T4Cell::Blank => " ",
            }
        )
    }
}

impl T4Cell {
    fn from_player(p: Player) -> T4Cell {
        match p {
            Player::P1 => T4Cell::X,
            Player::P2 => T4Cell::O,
        }
    }
}

#[derive(Clone)]
struct T2Board {
    cells: [T4Cell; 9],
    winning_piece: T4Cell,
}

impl T2Board {
    fn new() -> Self {
        T2Board {
            cells: [
                T4Cell::Blank,
                T4Cell::Blank,
                T4Cell::Blank,
                T4Cell::Blank,
                T4Cell::Blank,
                T4Cell::Blank,
                T4Cell::Blank,
                T4Cell::Blank,
                T4Cell::Blank,
            ],
            winning_piece: T4Cell::Blank,
        }
    }

    fn full(&self) -> bool {
        self.cells.iter().all(|c| *c != T4Cell::Blank)
    }

    fn valid(&self, place: u8) -> bool {
        place < 9 && self.cells[place as usize] == T4Cell::Blank
    }

    /// Returns whether the move happened
    fn play(&mut self, place: u8, player: Player) -> bool {
        if place < 9 && self.cells[place as usize] == T4Cell::Blank {
            self.cells[place as usize] = T4Cell::from_player(player);
            if self.winning_piece == T4Cell::Blank && self.has_won_p(player) {
                self.winning_piece = T4Cell::from_player(player)
            }
            true
        } else {
            false
        }
    }

    fn blanks(&self) -> Vec<u8> {
        (0..9)
            .filter(|i| self.cells[*i as usize] == T4Cell::Blank)
            .collect()
    }

    fn has_won_p(&self, player: Player) -> bool {
        let p = T4Cell::from_player(player);
        if self.cells[0] == p && self.cells[1] == p && self.cells[2] == p {
            return true;
        }
        if self.cells[3] == p && self.cells[4] == p && self.cells[5] == p {
            return true;
        }
        if self.cells[6] == p && self.cells[7] == p && self.cells[8] == p {
            return true;
        }
        if self.cells[0] == p && self.cells[3] == p && self.cells[6] == p {
            return true;
        }
        if self.cells[1] == p && self.cells[4] == p && self.cells[7] == p {
            return true;
        }
        if self.cells[2] == p && self.cells[5] == p && self.cells[8] == p {
            return true;
        }
        if self.cells[0] == p && self.cells[4] == p && self.cells[8] == p {
            return true;
        }
        if self.cells[2] == p && self.cells[4] == p && self.cells[6] == p {
            return true;
        }
        false
    }
}

#[derive(Clone)]
struct T4Board {
    boards: [T2Board; 9],
    next_player: Player,
    next_board: Option<u8>,
    winner: T4Cell,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct T4Move {
    macro_: u8,
    micro: u8,
}

impl T4Move {
    fn new(macro_: u8, micro: u8) -> Self {
        T4Move { macro_, micro }
    }
}

impl T4Board {
    fn new() -> Self {
        T4Board {
            boards: [
                T2Board::new(),
                T2Board::new(),
                T2Board::new(),
                T2Board::new(),
                T2Board::new(),
                T2Board::new(),
                T2Board::new(),
                T2Board::new(),
                T2Board::new(),
            ],
            next_player: Player::P1,
            next_board: None,
            winner: T4Cell::Blank,
        }
    }

    /// Returns validity
    fn play(&mut self, place: T4Move) -> bool {
        if self.next_board.map(|b| b == place.macro_).unwrap_or(true) {
            let valid = self.boards[place.macro_ as usize].play(place.micro, self.next_player);
            if valid {
                if self.has_won_p(self.next_player) {
                    self.winner = T4Cell::from_player(self.next_player);
                }
                self.next_player = self.next_player.other();
                self.next_board = if !self.boards[place.micro as usize].full() {
                    Some(place.micro)
                } else {
                    None
                }
            }
            valid
        } else {
            false
        }
    }

    fn valid(&self, place: T4Move) -> bool {
        self.next_board
            .map(|b| b == place.macro_)
            .unwrap_or(place.macro_ < 9)
            && self.boards[place.macro_ as usize].valid(place.micro)
    }

    fn full(&self) -> bool {
        self.boards.iter().all(|b| b.full())
    }

    fn has_won_p(&self, player: Player) -> bool {
        let p = T4Cell::from_player(player);
        if self.boards[0].winning_piece == p && self.boards[1].winning_piece == p
            && self.boards[2].winning_piece == p
        {
            return true;
        }
        if self.boards[3].winning_piece == p && self.boards[4].winning_piece == p
            && self.boards[5].winning_piece == p
        {
            return true;
        }
        if self.boards[6].winning_piece == p && self.boards[7].winning_piece == p
            && self.boards[8].winning_piece == p
        {
            return true;
        }
        if self.boards[0].winning_piece == p && self.boards[3].winning_piece == p
            && self.boards[6].winning_piece == p
        {
            return true;
        }
        if self.boards[1].winning_piece == p && self.boards[4].winning_piece == p
            && self.boards[7].winning_piece == p
        {
            return true;
        }
        if self.boards[2].winning_piece == p && self.boards[5].winning_piece == p
            && self.boards[8].winning_piece == p
        {
            return true;
        }
        if self.boards[0].winning_piece == p && self.boards[4].winning_piece == p
            && self.boards[8].winning_piece == p
        {
            return true;
        }
        if self.boards[2].winning_piece == p && self.boards[4].winning_piece == p
            && self.boards[6].winning_piece == p
        {
            return true;
        }
        false
    }
}

impl fmt::Display for T4Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for macro_row in [0, 1, 2usize].iter() {
            for micro_row in [0, 1, 2usize].iter() {
                for macro_col in [0, 1, 2usize].iter() {
                    write!(
                        f,
                        "{}",
                        self.boards[3 * macro_row + macro_col].cells[3 * micro_row + 0]
                    )?;
                    write!(
                        f,
                        "{}",
                        self.boards[3 * macro_row + macro_col].cells[3 * micro_row + 1]
                    )?;
                    write!(
                        f,
                        "{}",
                        self.boards[3 * macro_row + macro_col].cells[3 * micro_row + 2]
                    )?;
                    if *macro_col != 2 {
                        write!(f, " | ")?;
                    }
                }
                if *macro_row == 1 {
                    write!(f, "     {}", self.boards[3 * micro_row + 0].winning_piece)?;
                    write!(f, "{}", self.boards[3 * micro_row + 1].winning_piece)?;
                    write!(f, "{}", self.boards[3 * micro_row + 2].winning_piece)?;
                }
                writeln!(f, "")?;
            }
            if *macro_row == 2 {
                writeln!(f, "")?;
            } else {
                writeln!(f, "----+-----+----")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct T4BoardIter {
    moves: std::vec::IntoIter<T4Move>,
}

impl Iterator for T4BoardIter {
    type Item = T4Move;
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.moves.size_hint()
    }
    fn next(&mut self) -> Option<Self::Item> {
        self.moves.next()
    }
}

impl ExactSizeIterator for T4BoardIter {}

impl Default for T4BoardIter {
    fn default() -> Self {
        T4BoardIter {
            moves: Vec::new().into_iter(),
        }
    }
}

impl State for T4Board {
    type Action = T4Move;
    type Actions = T4BoardIter;

    fn initial() -> Self {
        T4Board::new()
    }

    fn next_player(&self) -> Player {
        self.next_player
    }

    fn do_action(&mut self, place: Self::Action) -> Outcome<Self::Actions> {
        self.play(place);
        if self.winner == T4Cell::from_player(self.next_player.other()) {
            Outcome::from_player(self.next_player.other())
        } else if self.full() {
            Outcome::Draw
        } else {
            let actions = self.valid_actions(self.next_player);
            Outcome::Actions(actions)
        }
    }

    fn valid_actions(&self, _: Player) -> Self::Actions {
        let v: Vec<T4Move> = if let Some(macro_) = self.next_board {
            self.boards[macro_ as usize]
                .blanks()
                .into_iter()
                .map(|micro| T4Move::new(macro_, micro))
                .collect()
        } else {
            (0..9)
                .flat_map(|macro_| {
                    self.boards[macro_]
                        .blanks()
                        .into_iter()
                        .map(move |micro| T4Move::new(macro_ as u8, micro))
                })
                .collect()
        };
        T4BoardIter {
            moves: v.into_iter(),
        }
    }

    fn has_won(&self, player: Player) -> bool {
        let p = T4Cell::from_player(player);
        if self.boards[0].winning_piece == p && self.boards[1].winning_piece == p
            && self.boards[2].winning_piece == p
        {
            return true;
        }
        if self.boards[3].winning_piece == p && self.boards[4].winning_piece == p
            && self.boards[5].winning_piece == p
        {
            return true;
        }
        if self.boards[6].winning_piece == p && self.boards[7].winning_piece == p
            && self.boards[8].winning_piece == p
        {
            return true;
        }
        if self.boards[0].winning_piece == p && self.boards[3].winning_piece == p
            && self.boards[6].winning_piece == p
        {
            return true;
        }
        if self.boards[1].winning_piece == p && self.boards[4].winning_piece == p
            && self.boards[7].winning_piece == p
        {
            return true;
        }
        if self.boards[2].winning_piece == p && self.boards[5].winning_piece == p
            && self.boards[8].winning_piece == p
        {
            return true;
        }
        if self.boards[0].winning_piece == p && self.boards[4].winning_piece == p
            && self.boards[8].winning_piece == p
        {
            return true;
        }
        if self.boards[2].winning_piece == p && self.boards[4].winning_piece == p
            && self.boards[6].winning_piece == p
        {
            return true;
        }
        false
    }
}

fn get_move(s: &T4Board) -> T4Move {
    let mut line = String::new();
    fn parse(line: &str) -> u8 {
        match line.trim() {
            "0" => 0,
            "1" => 1,
            "2" => 2,
            "3" => 3,
            "4" => 4,
            "5" => 5,
            "6" => 6,
            "7" => 7,
            "8" => 8,
            _ => 9,
        }
    }
    loop {
        println!("enter a macro board: ");
        io::stdin().read_line(&mut line).unwrap();
        let macro_ = parse(line.as_str());
        line.clear();
        println!("enter a micro board: ");
        io::stdin().read_line(&mut line).unwrap();
        let micro = parse(line.as_str());
        line.clear();
        let m = T4Move::new(macro_, micro);
        if !s.valid(m) {
            println!("Invalid move!");
        } else {
            return m;
        }
    }
}

#[allow(dead_code)]
fn mcts(thinking_time: usize) {
    let mut board = T4Board::initial();
    let mut mctree = MCTree::new(board.clone(), Player::P2, Player::P1);
    mctree.search_for(thinking_time);
    println!("{}", board);
    loop {
        let user_col = get_move(&board);
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
        println!("The AI played move {:?}", ai_col);
        println!(
            " it has played {} games from this position",
            mctree.root.visits()
        );
        println!(
            " and it believes it will win with p = {}",
            mctree.root.value()
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
    let thinking_time = env::args()
        .nth(1)
        .and_then(|a| usize::from_str(&a).ok())
        .unwrap_or(3000);
    mcts(thinking_time)
}
