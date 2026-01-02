use crate::{
    quadtree::{NodeKind, NodeRef},
    universe::Universe,
};
use web_sys::CanvasRenderingContext2d;

pub const DEFAULT_CELL_SIZE: f64 = 20.0;

#[derive(Clone)]
pub struct GolCanvas {
    pub ctx: CanvasRenderingContext2d,
    pub origin: (f64, f64),
    pub cell_size: f64,
}
impl GolCanvas {
    pub fn new(ctx: CanvasRenderingContext2d) -> Self {
        let mut gc = GolCanvas {
            ctx,
            origin: (0.0, 0.0),
            cell_size: DEFAULT_CELL_SIZE,
        };
        gc.set_center(0.0, 0.0);
        gc
    }
    pub fn width(&self) -> f64 {
        self.ctx.canvas().unwrap().width() as f64 / self.cell_size
    }
    pub fn height(&self) -> f64 {
        self.ctx.canvas().unwrap().height() as f64 / self.cell_size
    }
    pub fn get_zoom(&self) -> f64 {
        self.cell_size / DEFAULT_CELL_SIZE
    }
    pub fn to_grid(&self, offset_x: f64, offset_y: f64) -> (f64, f64) {
        (
            self.origin.0 + (offset_x / self.cell_size),
            self.origin.1 + (offset_y / self.cell_size),
        )
    }
    pub fn get_center(&self) -> (f64, f64) {
        (
            self.origin.0 + self.width() / 2.0,
            self.origin.1 + self.height() / 2.0,
        )
    }
    pub fn set_center(&mut self, x: f64, y: f64) {
        self.origin = (x - self.width() / 2.0, y - self.height() / 2.0);
    }
    pub fn zoom_at(&mut self, factor: f64, x: f64, y: f64) {
        self.cell_size *= factor;
        let f = 1.0 - 1.0 / factor;
        self.origin.0 += (x - self.origin.0) * f;
        self.origin.1 += (y - self.origin.1) * f;
    }
    pub fn zoom_at_center(&mut self, factor: f64) {
        let (x, y) = self.get_center();
        self.zoom_at(factor, x, y);
    }
    pub fn set_cell_size_at(&mut self, cell_size: f64, x: f64, y: f64) {
        let factor = cell_size / self.cell_size;
        self.zoom_at(factor, x, y);
    }
    pub fn fit_rect(&mut self, left: f64, top: f64, width: f64, height: f64) {
        let (canvas_width, canvas_height) = (
            self.ctx.canvas().unwrap().width() as f64,
            self.ctx.canvas().unwrap().height() as f64,
        );
        let (cell_width, cell_height) = (canvas_width / width, canvas_height / height);
        self.cell_size = cell_width.min(cell_height);
        self.set_center(left + width / 2.0, top + height / 2.0);
    }
    pub fn clear(&self) {
        let canvas = self.ctx.canvas().unwrap();
        self.ctx.set_fill_style_str("black");
        self.ctx
            .fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    }
    fn fill_rect(&self, left: f64, top: f64, width: f64, height: f64) {
        let (x, y) = (
            ((left) * self.cell_size).round(),
            ((top) * self.cell_size).round(),
        );
        let (actual_width, actual_height) = (
            ((left + width) * self.cell_size - x).round(),
            ((top + height) * self.cell_size - y).round(),
        );
        self.ctx.fill_rect(x, y, actual_width, actual_height);
    }
    pub fn draw_node(&self, universe: &Universe, node_ref: NodeRef, top: f64, left: f64) {
        let node = universe.arena.get(node_ref);
        if node.population == 0 {
            return;
        }

        let half = (1i64 << (node.level - 1)) as f64;
        let (bottom, right) = (top + 2.0 * half, left + 2.0 * half);
        if bottom < 0.0 || right < 0.0 || top > self.height() || left > self.width() {
            return;
        }

        if 2.0 * half * self.cell_size < 2.0 {
            self.ctx.set_fill_style_str(&format!(
                "rgba(255, 255, 255, {})",
                (node.population as f64 / (4.0 * half * half)).max(0.5)
            ));
            self.fill_rect(left, top, 2.0 * half, 2.0 * half);
            return;
        }

        self.ctx.set_fill_style_str("white");
        match &node.data {
            NodeKind::Leaf(leaf) => {
                for (y, row) in leaf.iter().enumerate() {
                    for (x, cell) in row.iter().enumerate() {
                        if *cell != 0 {
                            self.fill_rect(left + x as f64, top + y as f64, 1.0, 1.0);
                        }
                    }
                }
            }
            NodeKind::Branch([nw, ne, sw, se]) => {
                self.draw_node(universe, *nw, top, left);
                self.draw_node(universe, *ne, top, left + half);
                self.draw_node(universe, *sw, top + half, left);
                self.draw_node(universe, *se, top + half, left + half);
            }
        };
    }
    pub fn draw_rect(&self, top: f64, left: f64, width: f64, height: f64, rect: Vec<Vec<u8>>) {
        self.ctx.set_fill_style_str("white");
        let w = width / rect.len() as f64;
        let h = height / rect[0].len() as f64;

        for (y, row) in rect.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if *cell == 1 {
                    self.fill_rect(left + w * x as f64, top + h * y as f64, width, height);
                }
            }
        }
    }
}
