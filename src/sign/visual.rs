//! Draw-pad for visual signatures.

#[derive(Default, Clone)]
pub struct SignaturePad {
    /// Polylines in pad coordinates (0..pad_w, 0..pad_h).
    pub strokes: Vec<Vec<(f32, f32)>>,
    current: Option<Vec<(f32, f32)>>,
    pub pad_w: u32,
    pub pad_h: u32,
}

impl SignaturePad {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            strokes: Vec::new(),
            current: None,
            pad_w: w,
            pad_h: h,
        }
    }

    pub fn clear(&mut self) {
        self.strokes.clear();
        self.current = None;
    }

    pub fn is_empty(&self) -> bool {
        self.strokes.is_empty() && self.current.is_none()
    }

    pub fn begin(&mut self, x: f32, y: f32) {
        self.current = Some(vec![(x, y)]);
    }

    pub fn drag(&mut self, x: f32, y: f32) {
        if let Some(ref mut s) = self.current {
            s.push((x, y));
        }
    }

    pub fn end(&mut self) {
        if let Some(s) = self.current.take() {
            if s.len() > 1 {
                self.strokes.push(s);
            }
        }
    }

    /// Rasterize strokes to transparent RGBA.
    pub fn to_rgba(&self) -> (u32, u32, Vec<u8>) {
        let w = self.pad_w.max(1);
        let h = self.pad_h.max(1);
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        let all = self
            .strokes
            .iter()
            .chain(self.current.iter())
            .cloned()
            .collect::<Vec<_>>();
        for stroke in &all {
            for pair in stroke.windows(2) {
                draw_line(
                    &mut rgba,
                    w,
                    h,
                    pair[0].0,
                    pair[0].1,
                    pair[1].0,
                    pair[1].1,
                    2,
                );
            }
        }
        (w, h, rgba)
    }

    /// Load an RGBA image as the signature (replaces strokes).
    #[allow(dead_code)]
    pub fn from_rgba(&mut self, width: u32, height: u32, _rgba: &[u8]) {
        self.pad_w = width;
        self.pad_h = height;
        // Keep as baked image via to_rgba override — store as single full-rect marker.
        // Caller should use stamp_image directly for imported images.
        self.strokes.clear();
        self.current = None;
    }
}

fn draw_line(rgba: &mut [u8], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, thickness: i32) {
    let steps = ((x1 - x0).abs().max((y1 - y0).abs()) as i32).max(1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = x0 + (x1 - x0) * t;
        let y = y0 + (y1 - y0) * t;
        for dy in -thickness..=thickness {
            for dx in -thickness..=thickness {
                let px = (x as i32 + dx).clamp(0, w as i32 - 1) as u32;
                let py = (y as i32 + dy).clamp(0, h as i32 - 1) as u32;
                let idx = ((py * w + px) * 4) as usize;
                rgba[idx] = 20;
                rgba[idx + 1] = 20;
                rgba[idx + 2] = 40;
                rgba[idx + 3] = 255;
            }
        }
    }
}
