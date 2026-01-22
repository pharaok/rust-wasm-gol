use leptos::prelude::*;

use crate::{
    app::{PatternResult, fetch_pattern},
    parse::rle,
    universe::{InsertMode, Universe},
};

pub const META_CELL_LEVEL: u8 = 11;
pub const META_CELL_SIZE: i64 = 1 << META_CELL_LEVEL;

const CORNERS_RLE: &str = r#"
x = 2058, y = 2058, rule = B3/S23
bo2054bo$obo2052bobo$bo2054bo2$4b2o2046b2o$4bo2048bo2047$4bo2048bo$4b
2o2046b2o2$bo2054bo$obo2052bobo$bo2054bo!
"#;

pub fn use_metapixels() -> Signal<Option<(PatternResult, PatternResult)>, LocalStorage> {
    let meta_on_rle =
        LocalResource::new(move || fetch_pattern("otcametapixelonb3s23.rle".to_owned()));
    let meta_off_rle =
        LocalResource::new(move || fetch_pattern("otcametapixeloffb3s23.rle".to_owned()));
    Signal::derive_local(move || meta_on_rle.get().zip(meta_off_rle.get()))
}

impl Universe {
    pub fn set_grid_meta(&mut self, grid: &[Vec<u8>], meta_on_rle: &str, meta_off_rle: &str) {
        let (height, width) = (grid.len() as i64, grid[0].len() as i64);
        let h = 1 << (width + 2).max(height + 2).ilog2();
        let (extra_width, extra_height) = (2 * h - width, 2 * h - height);

        let off_points = rle::iter_alive(meta_off_rle)
            .unwrap()
            .map(|p| (p.0 - 5, p.1 - 5))
            .collect::<Vec<_>>();

        let half = 1i64 << (self.level() - 1);
        self.set_points(
            &off_points,
            -half,
            -half,
            half - 1,
            half - 1,
            &InsertMode::Copy,
        );
        let meta_off_ref = self.get_node(0, 0, META_CELL_LEVEL);
        self.clear();

        let on_points = rle::iter_alive(meta_on_rle)
            .unwrap()
            .map(|p| (p.0 - 5, p.1 - 5))
            .collect::<Vec<_>>();
        self.set_points(
            &on_points,
            -half,
            -half,
            half - 1,
            half - 1,
            &InsertMode::Copy,
        );
        let meta_on_ref = self.get_node(0, 0, META_CELL_LEVEL);
        self.clear();

        for y in -h..h {
            for x in -h..h {
                let (i, j) = ((y + h - extra_height / 2), (x + h - extra_width / 2));
                self.set_node(
                    x * META_CELL_SIZE,
                    y * META_CELL_SIZE,
                    META_CELL_LEVEL,
                    if grid
                        .get(i as usize)
                        .and_then(|row| row.get(j as usize))
                        .is_some_and(|&cell| cell != 0)
                    {
                        meta_on_ref
                    } else {
                        meta_off_ref
                    },
                );
            }
        }
        let corner_points = rle::iter_alive(CORNERS_RLE).unwrap().collect::<Vec<_>>();
        for dy in -h..h {
            for dx in -h..h {
                self.set_points(
                    &corner_points
                        .iter()
                        .map(|(x, y)| (x + dx * META_CELL_SIZE - 5, y + dy * META_CELL_SIZE - 5))
                        .collect::<Vec<_>>(),
                    dx * META_CELL_SIZE - 5,
                    dy * META_CELL_SIZE - 5,
                    (dx + 1) * META_CELL_SIZE + 5,
                    (dy + 1) * META_CELL_SIZE + 5,
                    &InsertMode::Or,
                );
            }
        }
    }
}
