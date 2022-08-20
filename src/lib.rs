mod utils;

use fixedbitset::FixedBitSet;
use js_sys::Math;
use std::vec;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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

    pub fn life_without_death() -> Rule {
        Rule {
            born: vec![3],
            survive: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
        }
    }

    pub fn modified_life_without_death() -> Rule {
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
pub struct PositionSet {
    positions: Vec<(u32, u32)>,
}

#[wasm_bindgen]
impl PositionSet {
    pub fn glider() -> PositionSet {
        PositionSet {
            positions: vec![(0, 0), (1, 1), (1, 2), (2, 0), (2, 1)],
        }
    }
    pub fn lwss() -> PositionSet {
        PositionSet {
            positions: vec![
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

    pub fn two_lwss_accident() -> PositionSet {
        let lwss = Self::lwss();
        let lwss_num_cells = 9;

        let mut positions = vec![(0, 0); lwss_num_cells * 2];
        for i in 0..lwss_num_cells {
            let (x, y) = lwss.positions[i];
            positions[i] = (x, y);
            positions[lwss_num_cells + i] = (y, x + 1);
        }

        Self { positions }
    }

    pub fn row(n: usize) -> PositionSet {
        let init_x: usize = 49;
        let init_y = 49 - n / 2;

        let mut positions = vec![(0, 0); n];
        for (i, position) in positions.iter_mut().enumerate() {
            position.0 = (init_x) as u32;
            position.1 = (init_y + i) as u32;
        }

        Self { positions }
    }

    pub fn column(n: usize) -> PositionSet {
        let init_x = 49 - n / 2;
        let init_y: usize = 49;

        let mut positions = vec![(0, 0); n];
        for (i, position) in positions.iter_mut().enumerate() {
            position.0 = (init_x + i) as u32;
            position.1 = (init_y) as u32;
        }

        Self { positions }
    }

    pub fn monster_without_death() -> PositionSet {
        PositionSet {
            positions: vec![(48, 49), (50, 48), (50, 50)],
        }
    }
}

#[wasm_bindgen]
pub struct Universe {
    rule: Rule,
    width: u32,
    height: u32,
    cells: FixedBitSet,
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

                next.set(
                    idx,
                    match (cell, live_neighbors) {
                        (false, n) => self.rule.born.contains(&n),
                        (true, n) => self.rule.survive.contains(&n),
                    },
                )
            }
        }

        self.cells = next;
    }

    pub fn new(initial: PositionSet, rule: Rule) -> Universe {
        let width = 99;
        let height = 99;

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        for i in 0..size {
            cells.set(
                i,
                initial
                    .positions
                    .contains(&(i as u32 / width, i as u32 % width)),
            )
        }

        Universe {
            rule,
            width,
            height,
            cells,
        }
    }

    pub fn random(life_chance: f64, rule: Rule) -> Universe {
        let width = 99;
        let height = 99;

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        for i in 0..size {
            let rand_num = Math::random();
            cells.set(i, rand_num < life_chance)
        }

        Universe {
            rule,
            width,
            height,
            cells,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }
}
