//! Opt-in local OCR via ocrs neural models.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use parking_lot::Mutex;
use rten::Model;
use rten_imageproc::BoundingRect;

use crate::error::{AppError, Result};

const DETECTION_URL: &str =
    "https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten";
const RECOGNITION_URL: &str =
    "https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten";

static ENGINE: OnceLock<Mutex<Option<OcrEngine>>> = OnceLock::new();

fn engine_slot() -> &'static Mutex<Option<OcrEngine>> {
    ENGINE.get_or_init(|| Mutex::new(None))
}

pub fn cache_dir() -> Result<PathBuf> {
    let base = dirs::cache_dir().ok_or_else(|| AppError::ocr("No cache directory"))?;
    let dir = base.join("pdf-opener").join("ocr");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn models_installed() -> bool {
    cache_dir()
        .map(|d| {
            d.join("text-detection.rten").is_file()
                && d.join("text-recognition.rten").is_file()
        })
        .unwrap_or(false)
}

pub fn model_paths() -> Result<(PathBuf, PathBuf)> {
    let dir = cache_dir()?;
    Ok((
        dir.join("text-detection.rten"),
        dir.join("text-recognition.rten"),
    ))
}

fn download_file(url: &str, dest: &Path) -> Result<()> {
    if dest.exists() {
        return Ok(());
    }
    let tmp = dest.with_extension("rten.part");
    let response = ureq::get(url)
        .call()
        .map_err(|e| AppError::ocr(format!("Download failed: {e}")))?;
    let mut reader = response.into_reader();
    let mut file = fs::File::create(&tmp)?;
    std::io::copy(&mut reader, &mut file)?;
    fs::rename(&tmp, dest)?;
    Ok(())
}

/// Download OCR models if missing. Safe to call repeatedly.
pub fn download_models(progress: &mut dyn FnMut(&str)) -> Result<()> {
    let (det, rec) = model_paths()?;
    progress("Downloading text detection model…");
    download_file(DETECTION_URL, &det)?;
    progress("Downloading text recognition model…");
    download_file(RECOGNITION_URL, &rec)?;
    progress("Models ready");
    *engine_slot().lock() = None;
    Ok(())
}

fn load_engine() -> Result<()> {
    if !models_installed() {
        return Err(AppError::ocr(
            "OCR models not installed — download them from the OCR panel",
        ));
    }
    let mut slot = engine_slot().lock();
    if slot.is_some() {
        return Ok(());
    }
    let (det_path, rec_path) = model_paths()?;
    let detection_model = Model::load_file(&det_path)
        .map_err(|e| AppError::ocr(format!("Load detection model: {e}")))?;
    let recognition_model = Model::load_file(&rec_path)
        .map_err(|e| AppError::ocr(format!("Load recognition model: {e}")))?;
    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })
    .map_err(|e| AppError::ocr(format!("Init OCR engine: {e}")))?;
    *slot = Some(engine);
    Ok(())
}

#[derive(Debug, Clone)]
pub struct OcrLine {
    pub text: String,
    /// Image / page coords with top-left origin (PDF points after OCR thread scale).
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Run OCR on an RGBA page image. Returns lines in pixel coordinates (top-left origin).
pub fn recognize_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<OcrLine>> {
    load_engine()?;
    let slot = engine_slot().lock();
    let engine = slot
        .as_ref()
        .ok_or_else(|| AppError::ocr("OCR engine not loaded"))?;

    let mut rgb = Vec::with_capacity((width * height * 3) as usize);
    for px in rgba.chunks_exact(4) {
        rgb.extend_from_slice(&px[..3]);
    }

    let source = ImageSource::from_bytes(&rgb, (width, height))
        .map_err(|e| AppError::ocr(format!("Image source: {e}")))?;
    let input = engine
        .prepare_input(source)
        .map_err(|e| AppError::ocr(format!("Prepare input: {e}")))?;
    let word_rects = engine
        .detect_words(&input)
        .map_err(|e| AppError::ocr(format!("Detect: {e}")))?;
    let line_rects = engine.find_text_lines(&input, &word_rects);
    let line_texts = engine
        .recognize_text(&input, &line_rects)
        .map_err(|e| AppError::ocr(format!("Recognize: {e}")))?;

    let mut out = Vec::new();
    for (rects, text_opt) in line_rects.iter().zip(line_texts.iter()) {
        let Some(text) = text_opt else { continue };
        let content = text.to_string();
        if content.trim().is_empty() {
            continue;
        }
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for r in rects {
            let b = r.bounding_rect();
            min_x = min_x.min(b.left() as f32);
            min_y = min_y.min(b.top() as f32);
            max_x = max_x.max(b.right() as f32);
            max_y = max_y.max(b.bottom() as f32);
        }
        if !min_x.is_finite() {
            continue;
        }
        out.push(OcrLine {
            text: content,
            x: min_x,
            y: max_y,
            width: (max_x - min_x).max(1.0),
            height: (max_y - min_y).max(8.0),
        });
    }
    Ok(out)
}
