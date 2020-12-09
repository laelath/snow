extern crate rand;
extern crate termion;

use rand::Rng;
use std::io::{Read, Write};
use std::{thread, time};
use termion::raw::IntoRawMode;
use termion::screen::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Snowflake {
    Type1,
    Type2,
    Type3,
}

impl Snowflake {
    fn create(variant: u8) -> Self {
        match variant {
            1 => Snowflake::Type1,
            2 => Snowflake::Type2,
            3 => Snowflake::Type3,
            _ => panic!("Invalid snowflake"),
        }
    }

    fn get_char(&self) -> char {
        match self {
            Snowflake::Type1 => '\u{2744}',
            Snowflake::Type2 => '\u{2745}',
            Snowflake::Type3 => '\u{2746}',
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Flake(Snowflake, u8),
    Stack(u8),
}

enum Action {
    Nop,
    Replace(Cell),
    Move(Snowflake, usize),
    Pile(u8, usize),
}

struct Snow {
    snow: Vec<Vec<Cell>>,
    rows: u16,
    cols: u16,
}

impl Snow {
    pub fn create(rows: u16, cols: u16) -> Self {
        Snow {
            snow: vec![vec![Cell::Empty; cols as usize]; rows as usize],
            rows: rows,
            cols: cols,
        }
    }

    fn update<R: Rng + ?Sized>(&mut self, rng: &mut R) -> () {
        // check for snowflakes landing at the bottom
        for cell in self.snow.last_mut().unwrap() {
            match *cell {
                Cell::Empty => (),
                Cell::Flake(variant, float) => {
                    if float == 0 {
                        *cell = Cell::Stack(1);
                    } else {
                        *cell = Cell::Flake(variant, float - 1);
                    }
                }
                Cell::Stack(_) => (),
            }
        }

        fn get_action(cell: Cell, col: usize, to: &Vec<Cell>) -> Action {
            match cell {
                Cell::Empty => Action::Nop,
                Cell::Flake(variant, float) => {
                    if float == 0 {
                        match to[col] {
                            Cell::Empty => Action::Move(variant, col),
                            Cell::Flake(_, _) => Action::Replace(Cell::Empty),
                            Cell::Stack(height) => {
                                if col > 0 && (to[col - 1] == Cell::Empty) {
                                    Action::Move(variant, col - 1)
                                } else if col < to.len() - 1 && (to[col + 1] == Cell::Empty) {
                                    Action::Move(variant, col + 1)
                                } else {
                                    let mut piles = [9, height, 9];

                                    if col > 0 {
                                        if let Cell::Stack(zheight) = to[col - 1] {
                                            piles[0] = zheight;
                                        }
                                    }
                                    if col < to.len() - 1 {
                                        if let Cell::Stack(zheight) = to[col + 1] {
                                            piles[2] = zheight;
                                        }
                                    }

                                    if piles[1] <= piles[0] && piles[1] <= piles[2] {
                                        if height < 8 {
                                            Action::Pile(height + 1, col)
                                        } else {
                                            Action::Replace(Cell::Stack(1))
                                        }
                                    } else if piles[2] <= piles[0] {
                                        Action::Pile(piles[2] + 1, col + 1)
                                    } else {
                                        Action::Pile(piles[0] + 1, col - 1)
                                    }
                                }
                            }
                        }
                    } else {
                        Action::Replace(Cell::Flake(variant, float - 1))
                    }
                }
                Cell::Stack(_) => Action::Nop,
            }
        }

        // float snowflakes down the screen
        for row in (0..(self.rows - 1) as usize).rev() {
            for col in 0..self.cols as usize {
                let (from, to) = self.snow.split_at_mut(row + 1);
                match get_action(from[row][col], col, &to[0]) {
                    Action::Nop => (),
                    Action::Replace(new) => from[row][col] = new,
                    Action::Move(variant, dest) => {
                        from[row][col] = Cell::Empty;
                        to[0][dest] = Cell::Flake(variant, rng.gen_range(1, 6));
                    }
                    Action::Pile(height, dest) => {
                        from[row][col] = Cell::Empty;
                        to[0][dest] = Cell::Stack(height);
                    }
                }
            }
        }

        // generate new snowflakes at the top
        for cell in self.snow.first_mut().unwrap() {
            if rng.gen_bool(1.0 / 500.0) {
                *cell = Cell::Flake(Snowflake::create(rng.gen_range(1, 4)), 0);
            }
        }
    }

    fn write<W: Write>(&self, screen: &mut W) {
        for row in 0..self.rows {
            let mut line = Vec::with_capacity(self.cols as usize);

            for cell in &self.snow[row as usize] {
                match cell {
                    Cell::Empty => line.push(' '),
                    Cell::Flake(variant, _) => line.push(variant.get_char()),
                    Cell::Stack(height) => line.push(match height {
                        1 => '\u{2581}',
                        2 => '\u{2582}',
                        3 => '\u{2583}',
                        4 => '\u{2584}',
                        5 => '\u{2585}',
                        6 => '\u{2586}',
                        7 => '\u{2587}',
                        8 => '\u{2588}',
                        _ => panic!("Invalid snowstack height"),
                    }),
                }
            }

            let s: String = line.into_iter().collect();
            write!(screen, "{}{}", termion::cursor::Goto(1, row + 1), s).unwrap();
        }
        screen.flush().unwrap();
    }
}

fn main() {
    let mut stdin = termion::async_stdin().bytes();
    let mut screen = AlternateScreen::from(std::io::stdout().into_raw_mode().unwrap());
    write!(screen, "{}", termion::cursor::Hide).unwrap();
    write!(screen, "{}", termion::clear::All).unwrap();

    let (cols, rows) = termion::terminal_size().unwrap();

    let mut snow = Snow::create(rows, cols);

    let mut rng = rand::thread_rng();

    loop {
        let b = stdin.next();
        if let Some(Ok(b'q')) = b {
            break;
        }

        snow.update(&mut rng);
        snow.write(&mut screen);

        thread::sleep(time::Duration::from_millis(50));
    }

    write!(screen, "{}", termion::cursor::Show).unwrap();
}
