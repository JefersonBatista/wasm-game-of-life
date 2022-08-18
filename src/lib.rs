mod utils;

use js_sys::Math;
use std::{fmt, vec};

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct Rule {
    born: Vec<u8>,
    survive: Vec<u8>,
}

#[wasm_bindgen]
impl Rule {
    pub fn freeze() -> Rule {
        Rule {
            born: vec![],
            survive: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
        }
    }

    pub fn modified_seeds() -> Rule {
        Rule {
            born: vec![1, 3],
            survive: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
        }
    }

    pub fn life() -> Rule {
        Rule {
            born: vec![3],
            survive: vec![2, 3],
        }
    }
}

#[wasm_bindgen]
pub struct CellSet {
    cells: Vec<(u32, u32)>,
}

#[wasm_bindgen]
impl CellSet {
    pub fn lwss() -> CellSet {
        CellSet {
            cells: vec![
                (46, 0),
                (46, 3),
                (47, 4),
                (48, 0),
                (48, 4),
                (49, 1),
                (49, 2),
                (49, 3),
                (49, 4),
            ],
        }
    }
}

#[wasm_bindgen]
pub struct Universe {
    rule: Rule,
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }
}

// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    (Cell::Dead, n) if self.rule.born.contains(&n) => Cell::Alive,
                    (Cell::Alive, n) if self.rule.survive.contains(&n) => Cell::Alive,
                    _ => Cell::Dead,
                };

                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }

    pub fn new(initial: CellSet, rule: Rule) -> Universe {
        let width = 99;
        let height = 99;

        let cells = (0..width * height)
            .map(|i| {
                if initial.cells.contains(&(i / width, i % width)) {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            rule,
            width,
            height,
            cells,
        }
    }

    pub fn random(rule: Rule) -> Universe {
        let width = 99;
        let height = 99;

        let cells = (0..width * height)
            .map(|_| {
                let rand_num = Math::random();
                if rand_num < 0.5 {
                    Cell::Dead
                } else {
                    Cell::Alive
                }
            })
            .collect();

        Universe {
            rule,
            width,
            height,
            cells,
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}
