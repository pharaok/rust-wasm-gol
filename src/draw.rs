use web_sys::{wasm_bindgen::JsValue, CanvasRenderingContext2d};

use crate::quadtree::{Node, NodeKind};

pub const DEFAULT_CELL_SIZE: f64 = 20.0;

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
        self.ctx.set_fill_style(&JsValue::from_str("black"));
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
    pub fn draw_node(&self, node: &Node, top: f64, left: f64) {
        if node.population == 0 {
            return;
        }

        let half = (1 << (node.level - 1)) as f64;
        let (bottom, right) = (top + 2.0 * half, left + 2.0 * half);
        if bottom < 0.0 || right < 0.0 || top > self.height() || left > self.width() {
            return;
        }

        if 2.0 * half * self.cell_size < 2.0 {
            self.ctx.set_fill_style(&JsValue::from_str(&format!(
                "rgba(255, 255, 255, {})",
                (node.population as f64 / (4.0 * half * half)).sqrt().sqrt()
            )));
            self.fill_rect(left, top, 2.0 * half, 2.0 * half);
            return;
        }

        self.ctx.set_fill_style(&JsValue::from_str("white"));
        match &node.node {
            NodeKind::Leaf(leaf) => {
                // guaranteed to be at leaf level
                for (i, row) in leaf.iter().enumerate() {
                    for (j, cell) in row.iter().enumerate() {
                        if *cell != 0 {
                            self.fill_rect(left + j as f64, top + i as f64, 1.0, 1.0);
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
