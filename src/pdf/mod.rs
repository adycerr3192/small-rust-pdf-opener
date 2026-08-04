//! PDF document session backed by MuPDF.

use std::path::{Path, PathBuf};

use mupdf::pdf::{
    InsertImageOptions, InsertPosition, PageImageSource, PageSelection, PdfDocument,
    PdfWriteOptions,
};
use mupdf::{Colorspace, Image, Matrix, Point, Rect, TextExtractOptions};

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressPreset {
    Fast,
    Balanced,
    Small,
}

impl CompressPreset {
    pub fn label(self) -> &'static str {
        match self {
            Self::Fast => "Fast",
            Self::Balanced => "Balanced",
            Self::Small => "Small",
        }
    }

    pub fn write_options(self) -> PdfWriteOptions {
        let mut opts = PdfWriteOptions::default();
        opts.set_compress(true)
            .set_compress_images(true)
            .set_compress_fonts(true)
            .set_clean(true);
        match self {
            Self::Fast => {
                opts.set_garbage_level(1);
            }
            Self::Balanced => {
                opts.set_garbage_level(2);
            }
            Self::Small => {
                opts.set_garbage_level(4);
                opts.set_sanitize(true);
            }
        }
        opts
    }
}

pub struct DocumentSession {
    pub path: Option<PathBuf>,
    doc: PdfDocument,
    pub dirty: bool,
}

impl DocumentSession {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let doc = PdfDocument::open(path)?;
        Ok(Self {
            path: Some(path.to_path_buf()),
            doc,
            dirty: false,
        })
    }

    pub fn page_count(&self) -> Result<usize> {
        Ok(self.doc.page_count()?.max(0) as usize)
    }

    pub fn page_size(&self, page: usize) -> Result<(f32, f32)> {
        let p = self.doc.load_pdf_page(page as i32)?;
        let b = p.bounds()?;
        Ok((b.width(), b.height()))
    }

    /// Render a page to RGBA8 pixels at the given zoom (1.0 = 72 dpi).
    pub fn render_page(&self, page: usize, zoom: f32) -> Result<RenderedPage> {
        let p = self.doc.load_pdf_page(page as i32)?;
        let ctm = Matrix::new_scale(zoom, zoom);
        let cs = Colorspace::device_rgb();
        let pixmap = p.to_pixmap(&ctm, &cs, true, true)?;
        let width = pixmap.width();
        let height = pixmap.height();
        let samples = pixmap.samples().to_vec();
        // MuPDF may return RGB or RGBA depending on alpha flag; we requested alpha.
        let n = pixmap.n() as usize;
        let rgba = if n == 4 {
            samples
        } else if n == 3 {
            let mut out = Vec::with_capacity(width as usize * height as usize * 4);
            for chunk in samples.chunks_exact(3) {
                out.extend_from_slice(chunk);
                out.push(255);
            }
            out
        } else {
            return Err(AppError::pdf(format!("unexpected pixmap components: {n}")));
        };
        Ok(RenderedPage {
            width,
            height,
            rgba,
        })
    }

    #[allow(dead_code)]
    pub fn extract_text(&self, page: usize) -> Result<String> {
        let p = self.doc.load_pdf_page(page as i32)?;
        Ok(p.text(TextExtractOptions::default())?)
    }

    pub fn search(&self, page: usize, needle: &str) -> Result<Vec<Rect>> {
        if needle.is_empty() {
            return Ok(Vec::new());
        }
        let p = self.doc.load_pdf_page(page as i32)?;
        let hits = p.search(needle, 100)?;
        Ok(hits.into_iter().map(|q| q.into()).collect())
    }

    pub fn delete_page(&mut self, page: usize) -> Result<()> {
        let count = self.page_count()?;
        if count <= 1 {
            return Err(AppError::msg("Cannot delete the only page"));
        }
        if page >= count {
            return Err(AppError::msg("Page out of range"));
        }
        self.doc
            .delete_pages(PageSelection::Pages(vec![page]))?;
        self.dirty = true;
        Ok(())
    }

    pub fn rotate_page(&mut self, page: usize, delta: i32) -> Result<()> {
        let mut p = self.doc.load_pdf_page(page as i32)?;
        let current = p.rotation().unwrap_or(0);
        let next = ((current + delta) % 360 + 360) % 360;
        p.set_rotation(next)?;
        self.dirty = true;
        Ok(())
    }

    pub fn move_page(&mut self, from: usize, to: usize) -> Result<()> {
        self.doc.move_page(from, to)?;
        self.dirty = true;
        Ok(())
    }

    pub fn set_crop(&mut self, page: usize, crop: Rect) -> Result<()> {
        let mut p = self.doc.load_pdf_page(page as i32)?;
        p.set_crop_box(crop)?;
        self.dirty = true;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn crop_box(&self, page: usize) -> Result<Rect> {
        let p = self.doc.load_pdf_page(page as i32)?;
        Ok(p.crop_box()?)
    }

    pub fn stamp_image(
        &mut self,
        page: usize,
        rgba: &[u8],
        width: u32,
        height: u32,
        rect: Rect,
    ) -> Result<()> {
        // Build an RGB pixmap-compatible Image via PNG encode round-trip for simplicity.
        let mut png_buf = Vec::new();
        {
            let encoder = image::codecs::png::PngEncoder::new(&mut png_buf);
            use image::ImageEncoder;
            encoder
                .write_image(rgba, width, height, image::ExtendedColorType::Rgba8)
                .map_err(|e| AppError::msg(e.to_string()))?;
        }
        let img = Image::from_bytes(&png_buf)?;
        let mut p = self.doc.load_pdf_page(page as i32)?;
        p.insert_image(
            &mut self.doc,
            rect,
            PageImageSource::Image(&img),
            InsertImageOptions {
                overlay: true,
                opacity: None,
                optional_content: None,
            },
        )?;
        self.dirty = true;
        Ok(())
    }

    /// Insert invisible OCR text (render mode 3) at page coordinates.
    pub fn add_ocr_text(
        &mut self,
        page: usize,
        lines: &[(String, f32, f32, f32)],
    ) -> Result<()> {
        use mupdf::shape::{Shape, TextOptions};
        let mut p = self.doc.load_pdf_page(page as i32)?;
        let mut shape = Shape::new(&mut p)?;
        for (text, x, y, fontsize) in lines {
            if text.trim().is_empty() {
                continue;
            }
            let opts = TextOptions {
                fontsize: *fontsize,
                render_mode: 3, // invisible
                ..Default::default()
            };
            shape.insert_text(Point::new(*x, *y), text, &opts)?;
        }
        shape.commit(&mut self.doc, true)?;
        self.dirty = true;
        Ok(())
    }

    pub fn save(&mut self) -> Result<()> {
        let path = self
            .path
            .clone()
            .ok_or_else(|| AppError::msg("No file path — use Save As"))?;
        self.save_as(&path, CompressPreset::Balanced.write_options())?;
        Ok(())
    }

    pub fn save_as(&mut self, path: impl AsRef<Path>, options: PdfWriteOptions) -> Result<()> {
        let path = path.as_ref();
        let path_str = path
            .to_str()
            .ok_or_else(|| AppError::msg("Path is not valid UTF-8"))?;
        self.doc.save_with_options(path_str, options)?;
        self.path = Some(path.to_path_buf());
        self.dirty = false;
        Ok(())
    }

    pub fn compress_save(
        &mut self,
        path: impl AsRef<Path>,
        preset: CompressPreset,
    ) -> Result<()> {
        self.save_as(path, preset.write_options())
    }

    /// Write document bytes for certificate signing (full rewrite, no incremental).
    pub fn write_bytes(&self, options: PdfWriteOptions) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.doc.write_to_with_options(&mut buf, options)?;
        Ok(buf)
    }

    #[allow(dead_code)]
    pub fn reload_from_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        let path = self.path.clone();
        let dirty = self.dirty;
        let doc = PdfDocument::from_bytes(bytes)?;
        self.doc = doc;
        self.path = path;
        self.dirty = dirty;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn duplicate_page_after(&mut self, page: usize) -> Result<()> {
        self.doc
            .copy_page(page, InsertPosition::After(page))?;
        self.dirty = true;
        Ok(())
    }
}

pub struct RenderedPage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl RenderedPage {
    pub fn to_color_image(&self) -> egui::ColorImage {
        egui::ColorImage::from_rgba_unmultiplied(
            [self.width as usize, self.height as usize],
            &self.rgba,
        )
    }
}

