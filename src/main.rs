extern crate ncurses;
extern crate rand;

use ncurses::*;
use rand::{thread_rng, Rng};
use std::{mem, str, thread, time};

fn main() {
    initscr();
    noecho();
    cbreak();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    nodelay(stdscr(), true);

    let mut cols = 0;
    let mut rows = 0;
    getmaxyx(stdscr(), &mut rows, &mut cols);

    let mut cols = cols as usize;
    let mut rows = rows as usize;

    let mut snow = vec![vec![' ' as u8; cols]; rows];
    let mut updt = vec![vec![0 as u8; cols]; rows];

    let mut rng = thread_rng();

    let mut ch = getch();
    while ch != 'q' as i32 {
        let mut new_cols = 0;
        let mut new_rows = 0;
        getmaxyx(stdscr(), &mut new_rows, &mut new_cols);
        if new_cols as usize != cols || new_rows as usize != rows {
            cols = new_cols as usize;
            rows = new_rows as usize;
            snow = vec![vec![' ' as u8; cols]; rows];
            updt = vec![vec![0 as u8; cols]; rows];
        }

        for row in (0..rows - 1).rev() {
            for col in 0..cols {
                let (from, to) = snow.split_at_mut(row + 1);
                if from[row][col] == '*' as u8 {
                    if updt[row][col] == 0 {
                        if to[0][col] == ' ' as u8 {
                            mem::swap(&mut from[row][col], &mut to[0][col]);
                            updt[row + 1][col] = rng.gen_range(1, 4);
                        } else if col != 0 && to[0][col - 1] == ' ' as u8 {
                            mem::swap(&mut from[row][col], &mut to[0][col - 1]);
                            updt[row + 1][col - 1] = rng.gen_range(1, 4);
                        } else if col != cols - 1 && to[0][col + 1] == ' ' as u8 {
                            mem::swap(&mut from[row][col], &mut to[0][col + 1]);
                            updt[row + 1][col + 1] = rng.gen_range(1, 4);
                        }
                    } else {
                        updt[row][col] -= 1;
                    }
                }
            }
        }

        for c in &mut snow[0] {
            if rng.gen_bool(1.0 / 500.0) {
                *c = '*' as u8;
            }
        }

        for i in 0..rows {
            mvprintw(i as i32, 0, unsafe { str::from_utf8_unchecked(&snow[i]) });
        }

        refresh();
        thread::sleep(time::Duration::from_millis(50));
        ch = getch();
    }

    endwin();
}
