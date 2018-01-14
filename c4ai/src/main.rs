extern crate mcts;

use std::fmt;
use std::io;
use std::env;
use mcts::*;

use std::str::FromStr;

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
    xs: u64,
    os: u64,
    next: Player,
}

impl fmt::Display for C4State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for r in 0..6 {
            write!(f, "|")?;
            write!(f, "{}", self.get(r, 0))?;
            for c in 1..7 {
                write!(f, " ")?;
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
        if ((self.os >> (row * 7 + col)) & 1) == 1 {
            C4Cell::O
        } else if ((self.xs >> (row * 7 + col)) & 1) == 1 {
            C4Cell::X
        } else {
            C4Cell::Blank
        }
    }
    fn play(&mut self, row: u8, col: u8, player: Player) {
        match player {
            Player::P1 => self.xs |= 1 << (row * 7 + col),
            Player::P2 => self.os |= 1 << (row * 7 + col),
        }
    }
    fn full(&self) -> bool {
        (self.xs | self.os).count_ones() == 42
    }
}

impl State for C4State {
    type Action = u8;
    type Actions = C4Actions;

    fn initial() -> Self {
        C4State {
            xs: 0,
            os: 0,
            next: Player::P1,
        }
    }

    fn next_player(&self) -> Player {
        self.next
    }

    fn do_action(&mut self, col: Self::Action) -> Outcome<Self::Actions> {
        for row in (0..6).rev() {
            if self.get(row, col) == C4Cell::Blank {
                let player = self.next;
                self.play(row, col, player);
                self.next = self.next.other();
                return if self.has_won(player) {
                    Outcome::from_player(player)
                } else if self.full() {
                    Outcome::Draw
                }else {
                    Outcome::Actions(self.valid_actions(self.next))
                };
            }
        }
        Outcome::Draw
    }

    fn valid_actions(&self, _: Player) -> Self::Actions {
        let mut bitvec = 0;
        if !self.has_won(Player::P1) && !self.has_won(Player::P2) {
            for i in (0..7).filter(|col| self.get(0, *col) == C4Cell::Blank) {
                bitvec |= 1u8 << i;
            }
        }
        C4Actions { bitvec }
    }

    fn has_won(&self, player: Player) -> bool {
        let streak = 4;
        let rows = 6;
        let cols = 7;
        let col_win = 0b0000000_0000000_0000001_0000001_0000001_0000001;
        let row_win = 0b0000000_0000000_0000000_0000000_0000000_0001111;
        let d1_win = 0b0000000_0000000_0001000_0000100_0000010_0000001;
        let d2_win = 0b0000000_0000000_0000001_0000010_0000100_0001000;
        let board = match player {
            Player::P1 => self.xs,
            Player::P2 => self.os,
        };


        // Column wins
        for s in 0..(cols * (rows - streak + 1)) {
            let win = col_win << s;
            if (board ^ win) & win == 0 {
                return true;
            }
        }

        // Check row wins
        for r in 0..(rows) {
            for c in 0..(cols - streak + 1) {
                let win = row_win << (r * 7 + c);
                if (board ^ win) & win == 0 {
                    return true;
                }
            }
        }

        // Check for diagonal wins
        for r in 0..(rows - streak + 1) {
            for c in 0..(cols - streak + 1) {
                let win = d1_win << (r * 7 + c);
                if (board ^ win) & win == 0 {
                    return true;
                }
                let win = d2_win << (r * 7 + c);
                if (board ^ win) & win == 0 {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Clone)]
struct C4Actions {
    bitvec: u8,
}

impl fmt::Debug for C4Actions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:07b}", self.bitvec)
    }
}

impl Default for C4Actions {
    fn default() -> Self {
        C4Actions { bitvec: 0 }
    }
}

impl Iterator for C4Actions {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        let ans = self.bitvec.trailing_zeros() as u8;
        if ans < 7 {
            self.bitvec &= !(1u8 << ans);
            Some(ans)
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let ones: usize = self.bitvec.count_ones() as usize;
        (ones, Some(ones))
    }
}

impl ExactSizeIterator for C4Actions {}


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
