use crate::{
    parse::rle::{self, PatternMetadata},
    quadtree::{Node, NodeKind, NodeRef},
    universe::Universe,
};
use web_sys::{CanvasRenderingContext2d, ImageData, wasm_bindgen::Clamped};

pub struct Canvas {
    pub ctx: CanvasRenderingContext2d,
    pub buffer: Vec<u8>,
}
impl Canvas {
    pub fn new(ctx: CanvasRenderingContext2d) -> Self {
        let (width, height) = (
            ctx.canvas().unwrap().width(),
            ctx.canvas().unwrap().height(),
        );
        let buffer_size = (width * height * 4) as usize;

        Self {
            ctx,
            buffer: vec![0; buffer_size],
        }
    }
    pub fn width(&self) -> u32 {
        self.ctx.canvas().unwrap().width()
    }
    pub fn height(&self) -> u32 {
        self.ctx.canvas().unwrap().height()
    }
    pub fn resize(&mut self) {
        let buffer_size = (self.width() * self.height() * 4) as usize;
        self.buffer = vec![0; buffer_size];
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0);
    }
    fn fill_rect(&mut self, viewport: &Viewport, x: f64, y: f64, width: f64, height: f64) {
        let (bottom, right) = (y + height, x + width);
        let cell_size = viewport.cell_size;
        if bottom < 0.0
            || right < 0.0
            || y > self.height() as f64 / cell_size
            || x > self.width() as f64 / cell_size
        {
            return;
        }

        let (canvas_x, canvas_y) = (
            (x * viewport.cell_size).round() as usize,
            (y * viewport.cell_size).round() as usize,
        );
        let (actual_width, actual_height) = (
            ((x + width) * viewport.cell_size - canvas_x as f64).round() as usize,
            ((y + height) * viewport.cell_size - canvas_y as f64).round() as usize,
        );

        let canvas_width = self.width() as usize;
        let stride = canvas_width * 4;
        let row_byte_start = canvas_x * 4;
        let row_byte_end = (canvas_x + actual_width).min(canvas_width) * 4;

        for yy in canvas_y..(canvas_y + actual_height) {
            let start_idx = yy * stride + row_byte_start;
            let end_idx = yy * stride + row_byte_end;
            if let Some(row_slice) = self.buffer.get_mut(start_idx..end_idx) {
                row_slice.fill(255);
            }
        }
    }
    pub fn draw(&self) {
        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.buffer),
            self.width(),
            self.height(),
        );
        if let Ok(data) = image_data {
            self.ctx.put_image_data(&data, 0.0, 0.0).unwrap();
        }
    }
}

#[derive(Clone)]
pub struct Viewport {
    pub origin: (f64, f64), // top left
    pub cell_size: f64,
}
impl Viewport {
    pub fn new() -> Self {
        Self {
            origin: (0.0, 0.0),
            cell_size: 20.0,
        }
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
    pub fn get_center(&self, canvas_width: f64, canvas_height: f64) -> (f64, f64) {
        (
            self.origin.0 + canvas_width / self.cell_size / 2.0,
            self.origin.1 + canvas_height / self.cell_size / 2.0,
        )
    }
    pub fn set_center(&mut self, x: f64, y: f64, canvas_width: f64, canvas_height: f64) {
        self.origin = (
            x - canvas_width / self.cell_size / 2.0,
            y - canvas_height / self.cell_size / 2.0,
        );
    }
    pub fn zoom_at(&mut self, factor: f64, x: f64, y: f64) {
        self.cell_size *= factor;
        let f = 1.0 - 1.0 / factor;
        self.origin.0 += (x - self.origin.0) * f;
        self.origin.1 += (y - self.origin.1) * f;
    }
    pub fn zoom_at_center(&mut self, factor: f64, canvas_width: f64, canvas_height: f64) {
        let (x, y) = self.get_center(canvas_width, canvas_height);
        self.zoom_at(factor, x, y);
    }
    pub fn set_cell_size_at(&mut self, cell_size: f64, x: f64, y: f64) {
        let factor = cell_size / self.cell_size;
        self.zoom_at(factor, x, y);
    }
    pub fn fit_rect(
        &mut self,
        left: f64,
        top: f64,
        width: f64,
        height: f64,
        canvas_width: f64,
        canvas_height: f64,
    ) {
        let (cell_width, cell_height) = (canvas_width / width, canvas_height / height);
        self.cell_size = cell_width.min(cell_height);
        self.set_center(
            left + width / 2.0,
            top + height / 2.0,
            canvas_width,
            canvas_height,
        );
    }
}

fn _draw_node(
    canvas: &mut Canvas,
    viewport: &Viewport,
    universe: &Universe,
    node_ref: NodeRef,
    x: f64,
    y: f64,
) {
    let node = universe.arena.get(node_ref);
    if node.population == 0 {
        return;
    }

    let half = (1i64 << (node.level - 1)) as f64;
    let (bottom, right) = (y + 2.0 * half, x + 2.0 * half);
    if bottom < 0.0
        || right < 0.0
        || y > canvas.height() as f64 / viewport.cell_size
        || x > canvas.width() as f64 / viewport.cell_size
    {
        return;
    }

    if 2.0 * half * viewport.cell_size < 2.0 {
        canvas.fill_rect(viewport, x, y, 2.0 * half, 2.0 * half);
        return;
    }

    match &node.data {
        NodeKind::Leaf(leaf) => {
            for (i, row) in leaf.iter().enumerate() {
                for (j, cell) in row.iter().enumerate() {
                    if *cell != 0 {
                        canvas.fill_rect(viewport, x + j as f64, y + i as f64, 1.0, 1.0);
                    }
                }
            }
        }
        NodeKind::Branch(children) => {
            for (i, child) in children.iter().enumerate() {
                let (ox, oy) = Node::get_child_offset(i, node.level);
                _draw_node(
                    canvas,
                    viewport,
                    universe,
                    *child,
                    x + ox as f64,
                    y + oy as f64,
                );
            }
        }
    };
}
pub fn draw_node(canvas: &mut Canvas, viewport: &Viewport, universe: &Universe) {
    let half = (1i64 << (universe.get_level() - 1)) as f64;
    _draw_node(
        canvas,
        viewport,
        universe,
        universe.root,
        -half - viewport.origin.0,
        -half - viewport.origin.1,
    );
    canvas.draw();
}
pub fn draw_rle(canvas: &mut Canvas, rle: String) {
    let (PatternMetadata { width, height, .. }, _) = rle::parse_metadata(&rle, "", "").unwrap();
    let mut vp = Viewport::new();
    vp.fit_rect(
        0.0,
        0.0,
        width as f64,
        height as f64,
        canvas.width() as f64,
        canvas.height() as f64,
    );
    vp.zoom_at_center(0.8, canvas.width() as f64, canvas.height() as f64);

    let l2 = (2.0 / vp.cell_size).log2() as usize;
    if l2 > 60 {
        return;
    }
    for (x, y) in rle::iter_alive(&rle).unwrap() {
        let (nx, ny) = (x >> l2 << l2, y >> l2 << l2);
        canvas.fill_rect(
            &vp,
            nx as f64 - vp.origin.0,
            ny as f64 - vp.origin.1,
            (1 << l2) as f64,
            (1 << l2) as f64,
        );
    }

    canvas.draw();
}
