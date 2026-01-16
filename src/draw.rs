use crate::{
    parse::rle::{self, PatternMetadata},
    quadtree::{Node, NodeKind, NodeRef},
    universe::Universe,
};
use web_sys::{CanvasRenderingContext2d, ImageData, wasm_bindgen::Clamped};

pub struct Canvas {
    pub ctx: CanvasRenderingContext2d,
    pub buffer: Vec<u8>,
    width: u32,
    height: u32,
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
            width,
            height,
        }
    }
    pub fn resize(&mut self) {
        self.width = self.ctx.canvas().unwrap().width();
        self.height = self.ctx.canvas().unwrap().height();
        let buffer_size = (self.width * self.height * 4) as usize;
        self.buffer = vec![0; buffer_size];
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0);
    }
    pub fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: u32) {
        // 0xRRGGBBAA
        let color_bytes: [u8; 4] = [
            ((color >> 24) & 0xFF) as u8,
            ((color >> 16) & 0xFF) as u8,
            ((color >> 8) & 0xFF) as u8,
            (color & 0xFF) as u8,
        ];

        let (x1, y1) = (x.max(0), y.max(0));
        let (x2, y2) = (
            (x + width).min(self.width as i32),
            (y + height).min(self.height as i32),
        );

        let stride = (self.width as i32) * 4;
        let row_byte_start = x1 * 4;
        let row_byte_end = x2 * 4;

        for y in y1..y2 {
            let start = y * stride + row_byte_start;
            let end = y * stride + row_byte_end;
            if let Some(row_slice) = self.buffer.get_mut((start as usize)..(end as usize)) {
                for pixel in row_slice.chunks_exact_mut(4) {
                    pixel.copy_from_slice(&color_bytes);
                }
            }
        }
    }
    pub fn fill_rect_with_viewport(
        &mut self,
        viewport: &Viewport,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color: u32,
    ) {
        let (bottom, right) = (y + height, x + width);
        let s = viewport.cell_size;
        if bottom < viewport.origin.1
            || right < viewport.origin.0
            || y > viewport.origin.1 + (self.height as f64 / s)
            || x > viewport.origin.0 + (self.width as f64 / s)
        {
            return;
        }

        let (c_x1, c_y1) = viewport.to_canvas_coords(x, y);
        let (c_x2, c_y2) = viewport.to_canvas_coords(x + width, y + height);

        self.fill_rect(c_x1, c_y1, c_x2 - c_x1, c_y2 - c_y1, color);
    }
    pub fn draw(&self) {
        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.buffer),
            self.width,
            self.height,
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

    pub fn to_world_coords(&self, offset_x: i32, offset_y: i32) -> (f64, f64) {
        (
            self.origin.0 + ((offset_x as f64) / self.cell_size),
            self.origin.1 + ((offset_y as f64) / self.cell_size),
        )
    }
    pub fn to_canvas_coords(&self, x: f64, y: f64) -> (i32, i32) {
        (
            ((x - self.origin.0) * self.cell_size).round() as i32,
            ((y - self.origin.1) * self.cell_size).round() as i32,
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
    x: i64,
    y: i64,
) {
    let node = universe.arena.get(node_ref);
    if node.population == 0 {
        return;
    }

    let half = (1i64 << (node.level - 1)) as f64;
    let (top, left) = (y as f64, x as f64);
    let (bottom, right) = (top + 2.0 * half, left + 2.0 * half);
    if right < viewport.origin.0
        || bottom < viewport.origin.1
        || left > viewport.origin.0 + canvas.width as f64 / viewport.cell_size
        || top > viewport.origin.1 + canvas.height as f64 / viewport.cell_size
    {
        return;
    }

    if 2.0 * half * viewport.cell_size < 2.0 {
        canvas.fill_rect_with_viewport(viewport, left, top, 2.0 * half, 2.0 * half, 0xFFFFFFFF);
        return;
    }

    match &node.data {
        NodeKind::Leaf(leaf) => {
            for (i, row) in leaf.iter().enumerate() {
                for (j, cell) in row.iter().enumerate() {
                    if *cell != 0 {
                        canvas.fill_rect_with_viewport(
                            viewport,
                            (x + j as i64) as f64,
                            (y + i as i64) as f64,
                            1.0,
                            1.0,
                            0xFFFFFFFF,
                        );
                    }
                }
            }
        }
        NodeKind::Branch(children) => {
            for (i, child) in children.iter().enumerate() {
                let (ox, oy) = Node::get_child_offset(i, node.level);
                _draw_node(canvas, viewport, universe, *child, x + ox, y + oy);
            }
        }
    };
}
pub fn draw_node(canvas: &mut Canvas, viewport: &Viewport, universe: &Universe) {
    let half = 1i64 << (universe.get_level() - 1);
    _draw_node(canvas, viewport, universe, universe.root, -half, -half);
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
        canvas.width as f64,
        canvas.height as f64,
    );
    vp.zoom_at_center(0.8, canvas.width as f64, canvas.height as f64);

    let l2 = (2.0 / vp.cell_size).log2() as usize;
    if l2 > 60 {
        return;
    }
    for (x, y) in rle::iter_alive(&rle).unwrap() {
        let (nx, ny) = (x >> l2 << l2, y >> l2 << l2);
        canvas.fill_rect_with_viewport(
            &vp,
            nx as f64,
            ny as f64,
            (1 << l2) as f64,
            (1 << l2) as f64,
            0xFFFFFFFF,
        );
    }

    canvas.draw();
}
