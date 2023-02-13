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
static GRID_WIDTH: u32 = 240;
static GRID_HEIGHT: u32 = 99;

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

    pub fn maze() -> Rule {
        Rule {
            born: vec![3],
            survive: vec![1, 2, 3, 4, 5],
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
        let middle = GRID_HEIGHT / 2;
        PositionSet {
            positions: vec![
                (middle - 3, 0),
                (middle - 3, 3),
                (middle - 2, 4),
                (middle - 1, 0),
                (middle - 1, 4),
                (middle, 1),
                (middle, 2),
                (middle, 3),
                (middle, 4),
            ],
        }
    }

    pub fn two_lwss_accident() -> PositionSet {
        let lwss = Self::lwss();
        let lwss_num_cells = lwss.positions.len();

        let mut positions = vec![(0, 0); lwss_num_cells * 2];
        for i in 0..lwss_num_cells {
            let (y, x) = lwss.positions[i];
            positions[i] = (y, x);
            positions[lwss_num_cells + i] = (x, y + 1);
        }

        Self { positions }
    }

    pub fn row(n: usize) -> PositionSet {
        let middle_x = GRID_WIDTH / 2;
        let middle_y = GRID_HEIGHT / 2;
        let init_x = middle_x - (n as u32) / 2;
        let init_y = middle_y;

        let mut positions = vec![(0, 0); n];
        for (i, position) in positions.iter_mut().enumerate() {
            position.0 = init_y;
            position.1 = init_x + (i as u32);
        }

        Self { positions }
    }

    pub fn column(n: usize) -> PositionSet {
        let middle_x = GRID_WIDTH / 2;
        let middle_y = GRID_HEIGHT / 2;
        let init_x = middle_x;
        let init_y = middle_y - (n as u32) / 2;

        let mut positions = vec![(0, 0); n];
        for (i, position) in positions.iter_mut().enumerate() {
            position.0 = init_y + (i as u32);
            position.1 = init_x;
        }

        Self { positions }
    }

    pub fn monster_without_death() -> PositionSet {
        let middle_x = GRID_WIDTH / 2;
        let middle_y = GRID_HEIGHT / 2;
        PositionSet {
            positions: vec![
                (middle_y - 1, middle_x),
                (middle_y + 1, middle_x - 1),
                (middle_y + 1, middle_x + 1),
            ],
        }
    }
}

impl PositionSet {
    pub fn empty() -> Self {
        PositionSet { positions: vec![] }
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

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.set(idx, true);
        }
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
        let width = GRID_WIDTH;
        let height = GRID_HEIGHT;

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
        let width = GRID_WIDTH;
        let height = GRID_HEIGHT;

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

    /// Set the width of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.cells.set_range(.., false);
    }

    /// Set the height of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells.set_range(.., false);
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells.set(idx, !self.cells[idx]);
    }
}
