mod util;
mod m3;

use std::fmt;
use wasm_bindgen::prelude::wasm_bindgen;

extern crate itertools;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct UniverseConfig {
    lower_alive_bound: u8,
    upper_alive_bound: u8,
    revive_count: u8,
}

#[wasm_bindgen]
impl UniverseConfig {
    pub fn new(lower_alive_bound: u8, upper_alive_bound: u8, revive_count: u8) -> UniverseConfig {
        UniverseConfig {
            lower_alive_bound,
            upper_alive_bound,
            revive_count,
        }
    }
}

#[wasm_bindgen]
pub struct Universe {
    conf: UniverseConfig,
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

#[wasm_bindgen]
impl Universe {
    fn pos(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    fn num_live_neighbors(&self, x: u32, y: u32) -> u8 {
        let mut count = 0;
        for delta_x in [self.height - 1, 0, 1].iter() {
            for delta_y in [self.width - 1, 0, 1].iter() {
                if *delta_x == 0 && *delta_y == 0 {
                    continue;
                }
                let neighbor_idx =
                    self.pos((delta_x + x) % self.width, (delta_y + y) % self.height);
                count += self.cells[neighbor_idx] as u8;
            }
        }
        count
    }

    pub fn new(width: u32, height: u32, conf: UniverseConfig) -> Universe {
        let cells: Vec<Cell> = (0..width * height)
            .map(|i| {
                if i % 3 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();
        Universe {
            conf,
            width,
            height,
            cells,
        }
    }

    pub fn update(&mut self) {
        let mut next = self.cells.clone();
        for y in 0..self.height {
            for x in 0..self.width {
                let write_head_pos = self.pos(x, y);
                let live_neighbors: u8 = self.num_live_neighbors(x, y);
                next[write_head_pos] = match (self.cells[write_head_pos], live_neighbors) {
                    (Cell::Alive, x)
                        if (x < self.conf.lower_alive_bound || x > self.conf.upper_alive_bound) =>
                    {
                        Cell::Dead
                    }
                    (Cell::Dead, x) if (x == self.conf.revive_count) => Cell::Alive,
                    (elsewise, _) => elsewise,
                }
            }
        }
        self.cells = next;
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn to_image_data(&self) -> Vec<u8> {
        let num_fields = 4;
        let out_len = self.cells.len() * num_fields;
        let mut out: Vec<u8> = Vec::with_capacity(out_len);
        for i in 0..self.cells.len() {
            let (r, g, b, a) = match self.cells[i] {
                Cell::Alive => (0, 0, 0, 255),
                Cell::Dead => (0, 0, 0, 0),
            };
            out.push(r);
            out.push(g);
            out.push(b);
            out.push(a);
        }
        out
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

#[wasm_bindgen]
pub fn greet() -> String {
    let conf = UniverseConfig {
        lower_alive_bound: 2,
        upper_alive_bound: 4,
        revive_count: 3,
    };
    let univ = Universe::new(10, 10, conf);
    univ.render()
}
