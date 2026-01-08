use crate::{
    parse::rle::{self, PatternMetadata},
    quadtree::{Node, NodeKind, NodeRef},
    universe::Universe,
};
use leptos::logging;
use web_sys::{CanvasRenderingContext2d, ImageData, wasm_bindgen::Clamped};

pub const DEFAULT_CELL_SIZE: f64 = 20.0;

#[derive(Clone)]
pub struct GolCanvas {
    pub ctx: CanvasRenderingContext2d,
    pub buffer: Vec<u8>,
    pub origin: (f64, f64),
    pub cell_size: f64,
    width: u32,
    height: u32,
}
impl GolCanvas {
    pub fn new(ctx: CanvasRenderingContext2d) -> Self {
        let (width, height) = (
            ctx.canvas().unwrap().width(),
            ctx.canvas().unwrap().height(),
        );
        let buffer_size = (width * height * 4) as usize;

        let mut gc = GolCanvas {
            ctx,
            buffer: vec![0; buffer_size],
            origin: (0.0, 0.0),
            cell_size: DEFAULT_CELL_SIZE,
            width,
            height,
        };
        gc.set_center(0.0, 0.0);
        gc
    }
    pub fn canvas_width(&self) -> u32 {
        self.width
    }
    pub fn canvas_height(&self) -> u32 {
        self.height
    }
    pub fn width(&self) -> f64 {
        self.canvas_width() as f64 / self.cell_size
    }
    pub fn height(&self) -> f64 {
        self.canvas_height() as f64 / self.cell_size
    }
    pub fn resize(&mut self) {
        self.width = self.ctx.canvas().unwrap().width();
        self.height = self.ctx.canvas().unwrap().height();
        let buffer_size = (self.canvas_width() * self.canvas_height() * 4) as usize;
        self.buffer = vec![0; buffer_size];
    }

    pub fn to_world_coords(&self, offset_x: f64, offset_y: f64) -> (f64, f64) {
        (
            self.origin.0 + (offset_x / self.cell_size),
            self.origin.1 + (offset_y / self.cell_size),
        )
    }
    pub fn to_canvas_coords(&self, x: f64, y: f64) -> (f64, f64) {
        (
            (x - self.origin.0) * self.cell_size,
            (y - self.origin.1) * self.cell_size,
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
        let (canvas_width, canvas_height) =
            (self.canvas_width() as f64, self.canvas_height() as f64);
        let (cell_width, cell_height) = (canvas_width / width, canvas_height / height);
        self.cell_size = cell_width.min(cell_height);
        self.set_center(left + width / 2.0, top + height / 2.0);
    }
    pub fn clear(&mut self) {
        self.buffer.fill(0);
    }
    fn fill_rect(&mut self, left: f64, top: f64, width: f64, height: f64) {
        let (bottom, right) = (top + height, left + width);
        if bottom < 0.0 || right < 0.0 || top > self.height() || left > self.width() {
            return;
        }

        let (x, y) = (
            ((left) * self.cell_size).round() as usize,
            ((top) * self.cell_size).round() as usize,
        );
        let (actual_width, actual_height) = (
            ((left + width) * self.cell_size - x as f64).round() as usize,
            ((top + height) * self.cell_size - y as f64).round() as usize,
        );

        let canvas_width = self.canvas_width() as usize;
        let stride = canvas_width * 4;
        let row_byte_start = x * 4;
        let row_byte_end = (x + actual_width).min(canvas_width) * 4;

        for yy in y..(y + actual_height) {
            let start_idx = yy * stride + row_byte_start;
            let end_idx = yy * stride + row_byte_end;
            if let Some(row_slice) = self.buffer.get_mut(start_idx..end_idx) {
                row_slice.fill(255);
            }
        }
    }
    pub fn _draw_node(&mut self, universe: &Universe, node_ref: NodeRef, left: f64, top: f64) {
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
            self.fill_rect(left, top, 2.0 * half, 2.0 * half);
            return;
        }

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
            NodeKind::Branch(children) => {
                for (i, child) in children.iter().enumerate() {
                    let (ox, oy) = Node::get_child_offset(i, node.level);
                    self._draw_node(universe, *child, left + ox as f64, top + oy as f64);
                }
            }
        };
    }
    pub fn draw_node(&mut self, universe: &Universe, left: f64, top: f64) {
        self._draw_node(universe, universe.root, left, top);

        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.buffer),
            self.canvas_width(),
            self.canvas_height(),
        )
        .unwrap();
        self.ctx.put_image_data(&image_data, 0.0, 0.0).unwrap();
    }
    pub fn draw_rle(&mut self, rle: String) {
        let (PatternMetadata { width, height, .. }, _) = rle::parse_metadata(&rle, "", "").unwrap();
        self.fit_rect(0.0, 0.0, width as f64, height as f64);
        self.zoom_at_center(0.8);

        let l2 = (2.0 / self.cell_size).log2() as usize;
        if l2 > 60 {
            return;
        }
        for (x, y) in rle::iter_alive(&rle).unwrap() {
            let (nx, ny) = (x >> l2 << l2, y >> l2 << l2);
            self.fill_rect(
                nx as f64 - self.origin.0,
                ny as f64 - self.origin.1,
                (1 << l2) as f64,
                (1 << l2) as f64,
            );
        }

        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.buffer),
            self.canvas_width(),
            self.canvas_height(),
        );
        if let Ok(data) = image_data {
            self.ctx.put_image_data(&data, 0.0, 0.0).unwrap();
        }
    }
}
