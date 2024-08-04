use web_sys::CanvasRenderingContext2d;

use crate::quadtree::{Node, NodeKind};

pub const DEFAULT_CELL_SIZE: f64 = 20.0;

pub struct GolCanvas {
    pub ctx: CanvasRenderingContext2d,
    pub ox: f64,
    pub oy: f64,
    pub cell_size: f64,
}
impl GolCanvas {
    pub fn new(ctx: CanvasRenderingContext2d) -> Self {
        let mut gc = GolCanvas {
            ctx,
            ox: 0.0,
            oy: 0.0,
            cell_size: DEFAULT_CELL_SIZE,
        };
        gc.ox = -gc.width() / 2.0;
        gc.oy = -gc.height() / 2.0;
        gc
    }
    pub fn width(&self) -> f64 {
        self.ctx.canvas().unwrap().width() as f64 / self.cell_size
    }
    pub fn height(&self) -> f64 {
        self.ctx.canvas().unwrap().height() as f64 / self.cell_size
    }
    pub fn zoom(&self) -> f64 {
        self.cell_size / DEFAULT_CELL_SIZE
    }
    pub fn to_grid(&self, offset_x: f64, offset_y: f64) -> (f64, f64) {
        (
            self.ox + (offset_x / self.cell_size),
            self.oy + (offset_y / self.cell_size),
        )
    }
    pub fn zoom_at(&mut self, factor: f64, x: f64, y: f64) {
        self.cell_size *= factor;
        let f = 1.0 - 1.0 / factor;
        self.ox += (x - self.ox) * f;
        self.oy += (y - self.oy) * f;
    }
    pub fn clear(&self) {
        let canvas = self.ctx.canvas().unwrap();
        self.ctx
            .clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    }
    pub fn draw_node(&self, node: &Node, top: f64, left: f64) {
        if node.population == 0 {
            return;
        }

        let half = (1 << (node.level - 1)) as f64;
        let (bottom, right) = (top + 2.0 * half, left + 2.0 * half);
        if bottom < 0.0 || right < 0.0 || top > self.height() || left > self.width() {
            return;
        }

        let cell_size = self.cell_size;

        match &node.node {
            NodeKind::Leaf(leaf) => {
                // guaranteed to be at leaf level
                for (i, row) in leaf.iter().enumerate() {
                    for (j, cell) in row.iter().enumerate() {
                        if *cell != 0 {
                            let (x, y) = (
                                ((left + j as f64) * cell_size).round(),
                                ((top + i as f64) * cell_size).round(),
                            );
                            // sharp edges without gaps between cells
                            let (actual_width, actual_height) = (
                                ((left + j as f64 + 1.0) * cell_size - x).round(),
                                ((top + i as f64 + 1.0) * cell_size - y).round(),
                            );
                            self.ctx.fill_rect(x, y, actual_width, actual_height);
                        }
                    }
                }
            }
            NodeKind::Branch([nw, ne, sw, se]) => {
                self.draw_node(&nw.borrow(), top, left);
                self.draw_node(&ne.borrow(), top, left + half);
                self.draw_node(&sw.borrow(), top + half, left);
                self.draw_node(&se.borrow(), top + half, left + half);
            }
        };
    }
}
