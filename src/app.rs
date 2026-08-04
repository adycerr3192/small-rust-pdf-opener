//! Main egui application.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};

use eframe::egui;
use mupdf::Rect as PdfRect;

use crate::error::AppError;
use crate::ocr;
use crate::pdf::{CompressPreset, DocumentSession};
use crate::sign::cert::{self, CertIdentity};
use crate::sign::visual::SignaturePad;

const PAGE_GAP: f32 = 16.0;
const TEXTURE_CACHE_RADIUS: isize = 2;

enum PageInteraction {
    CropEnd {
        page: usize,
        start: egui::Pos2,
        end: egui::Pos2,
        rect: egui::Rect,
    },
    SignClick {
        page: usize,
        pos: egui::Pos2,
        rect: egui::Rect,
    },
    CertEnd {
        page: usize,
        start: egui::Pos2,
        end: egui::Pos2,
        rect: egui::Rect,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolMode {
    View,
    Crop,
    VisualSign,
    CertSign,
}

enum BackgroundMsg {
    OcrProgress(String),
    OcrDone(Result<Vec<ocr::OcrLine>, String>),
    OcrModelsDone(Result<(), String>),
    Error(String), // reserved for generic bg failures
}

pub struct PdfApp {
    session: Option<DocumentSession>,
    page: usize,
    zoom: f32,
    fit_width: bool,
    mode: ToolMode,
    search_query: String,
    search_hits: Vec<PdfRect>,
    status: String,
    error: Option<String>,
    show_compress: bool,
    compress_preset: CompressPreset,
    show_ocr: bool,
    /// Cached page textures keyed by page index; invalidated when zoom changes.
    page_textures: HashMap<usize, egui::TextureHandle>,
    texture_zoom: f32,
    /// When set, viewer scrolls this page into view once.
    scroll_to_page: Option<usize>,
    // Crop
    crop_start: Option<egui::Pos2>,
    crop_end: Option<egui::Pos2>,
    crop_page: Option<usize>,
    // Visual sign
    sig_pad: SignaturePad,
    placing_sig: bool,
    imported_sig: Option<(u32, u32, Vec<u8>)>,
    // Cert sign
    cert_identity: Option<CertIdentity>,
    cert_password: String,
    cert_path: Option<PathBuf>,
    cert_reason: String,
    placing_cert: bool,
    cert_place: Option<(egui::Pos2, egui::Pos2)>,
    cert_page: Option<usize>,
    // Background work
    bg_tx: Sender<BackgroundMsg>,
    bg_rx: Receiver<BackgroundMsg>,
    busy: bool,
}

impl Default for PdfApp {
    fn default() -> Self {
        let (bg_tx, bg_rx) = mpsc::channel();
        Self {
            session: None,
            page: 0,
            zoom: 1.25,
            fit_width: true,
            mode: ToolMode::View,
            search_query: String::new(),
            search_hits: Vec::new(),
            status: "Open a PDF to begin".into(),
            error: None,
            show_compress: false,
            compress_preset: CompressPreset::Balanced,
            show_ocr: false,
            page_textures: HashMap::new(),
            texture_zoom: 0.0,
            scroll_to_page: None,
            crop_start: None,
            crop_end: None,
            crop_page: None,
            sig_pad: SignaturePad::new(400, 150),
            placing_sig: false,
            imported_sig: None,
            cert_identity: None,
            cert_password: String::new(),
            cert_path: None,
            cert_reason: "Signed with PDF Opener".into(),
            placing_cert: false,
            cert_place: None,
            cert_page: None,
            bg_tx,
            bg_rx,
            busy: false,
        }
    }
}

impl PdfApp {
    pub fn new(cc: &eframe::CreationContext<'_>, initial: Option<PathBuf>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut app = Self::default();
        if let Some(path) = initial {
            app.open_path(path);
        }
        app
    }

    fn open_path(&mut self, path: PathBuf) {
        match DocumentSession::open(&path) {
            Ok(session) => {
                self.status = format!("Opened {}", path.display());
                self.page = 0;
                self.session = Some(session);
                self.invalidate_textures();
                self.scroll_to_page = Some(0);
                self.search_hits.clear();
                self.error = None;
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    fn open_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            .pick_file()
        {
            self.open_path(path);
        }
    }

    fn save(&mut self) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        match session.save() {
            Ok(()) => self.status = "Saved".into(),
            Err(e) => {
                // Fall back to Save As if no path / incremental issues
                self.error = Some(e.to_string());
                self.save_as();
            }
        }
    }

    fn save_as(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            .set_file_name("document.pdf")
            .save_file()
        else {
            return;
        };
        if let Some(session) = self.session.as_mut() {
            match session.save_as(&path, CompressPreset::Balanced.write_options()) {
                Ok(()) => self.status = format!("Saved {}", path.display()),
                Err(e) => self.error = Some(e.to_string()),
            }
        }
    }

    fn invalidate_textures(&mut self) {
        self.page_textures.clear();
        self.texture_zoom = 0.0;
    }

    fn update_zoom(&mut self, available_width: f32) {
        let Some(session) = self.session.as_ref() else {
            return;
        };
        if self.fit_width {
            // Use first page (or current) width for fit-width zoom.
            let page = self.page.min(session.page_count().unwrap_or(1).saturating_sub(1));
            if let Ok((pw, _)) = session.page_size(page) {
                if pw > 0.0 {
                    let zoom = (available_width / pw).clamp(0.25, 4.0);
                    if (zoom - self.zoom).abs() > 0.001 {
                        self.zoom = zoom;
                        self.invalidate_textures();
                    }
                }
            }
        }
    }

    fn ensure_page_texture(&mut self, ctx: &egui::Context, page: usize) {
        let zoom = self.zoom;
        if (self.texture_zoom - zoom).abs() > 0.001 {
            self.page_textures.clear();
            self.texture_zoom = zoom;
        }
        if self.page_textures.contains_key(&page) {
            return;
        }
        let Some(session) = self.session.as_ref() else {
            return;
        };
        match session.render_page(page, zoom) {
            Ok(rendered) => {
                let img = rendered.to_color_image();
                let tex = ctx.load_texture(
                    format!("page-{page}-{zoom:.3}"),
                    img,
                    egui::TextureOptions::LINEAR,
                );
                self.page_textures.insert(page, tex);
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    fn prune_texture_cache(&mut self) {
        let current = self.page as isize;
        self.page_textures.retain(|&p, _| {
            let d = (p as isize - current).abs();
            d <= TEXTURE_CACHE_RADIUS + 1
        });
    }

    fn page_display_size(&self, page: usize) -> egui::Vec2 {
        let zoom = self.zoom;
        if let Some(tex) = self.page_textures.get(&page) {
            return tex.size_vec2();
        }
        if let Some(session) = self.session.as_ref() {
            if let Ok((w, h)) = session.page_size(page) {
                return egui::vec2(w * zoom, h * zoom);
            }
        }
        egui::vec2(600.0, 800.0)
    }

    fn go_to_page(&mut self, page: usize) {
        let count = self.page_count();
        if count == 0 {
            return;
        }
        let page = page.min(count - 1);
        self.page = page;
        self.scroll_to_page = Some(page);
        self.search_hits.clear();
    }

    fn page_count(&self) -> usize {
        self.session
            .as_ref()
            .and_then(|s| s.page_count().ok())
            .unwrap_or(0)
    }

    fn poll_background(&mut self) {
        while let Ok(msg) = self.bg_rx.try_recv() {
            match msg {
                BackgroundMsg::OcrProgress(s) => self.status = s,
                BackgroundMsg::OcrModelsDone(res) => {
                    self.busy = false;
                    match res {
                        Ok(()) => self.status = "OCR models downloaded".into(),
                        Err(e) => self.error = Some(e),
                    }
                }
                BackgroundMsg::OcrDone(res) => {
                    self.busy = false;
                    match res {
                        Ok(lines) => self.apply_ocr_lines(lines),
                        Err(e) => self.error = Some(e),
                    }
                }
                BackgroundMsg::Error(e) => {
                    self.busy = false;
                    self.error = Some(e);
                }
            }
        }
    }

    fn apply_ocr_lines(&mut self, lines: Vec<ocr::OcrLine>) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        // Lines arrive in PDF points with top-left origin (OCR thread already unscaled).
        let Ok((_, page_h)) = session.page_size(self.page) else {
            return;
        };
        let mapped: Vec<(String, f32, f32, f32)> = lines
            .into_iter()
            .map(|l| {
                let pdf_x = l.x;
                let pdf_y = page_h - l.y;
                let fontsize = l.height.clamp(6.0, 36.0);
                (l.text, pdf_x, pdf_y, fontsize)
            })
            .collect();
        match session.add_ocr_text(self.page, &mapped) {
            Ok(()) => {
                self.status = format!("OCR added {} text lines", mapped.len());
                self.invalidate_textures();
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    fn start_ocr_page(&mut self) {
        if !ocr::models_installed() {
            self.error = Some("Download OCR models first (OCR panel)".into());
            self.show_ocr = true;
            return;
        }
        let Some(session) = self.session.as_ref() else {
            return;
        };
        let page = self.page;
        let zoom = 2.0; // Higher res for OCR
        let rendered = match session.render_page(page, zoom) {
            Ok(r) => r,
            Err(e) => {
                self.error = Some(e.to_string());
                return;
            }
        };
        self.busy = true;
        self.status = "Running OCR…".into();
        let tx = self.bg_tx.clone();
        std::thread::spawn(move || {
            let result = ocr::recognize_rgba(rendered.width, rendered.height, &rendered.rgba)
                .map_err(|e| e.to_string());
            // Scale line coords from OCR zoom to current display is handled in apply
            // using self.zoom — fix: store OCR zoom in lines by scaling to page space here.
            let result = result.map(|lines| {
                lines
                    .into_iter()
                    .map(|mut l| {
                        // Convert to zoom=1 pixel space (= PDF points from top)
                        l.x /= zoom;
                        l.y /= zoom;
                        l.height /= zoom;
                        l
                    })
                    .collect::<Vec<_>>()
            });
            let _ = tx.send(BackgroundMsg::OcrDone(result));
        });
    }

    fn download_ocr_models(&mut self) {
        self.busy = true;
        self.status = "Downloading OCR models…".into();
        let tx = self.bg_tx.clone();
        std::thread::spawn(move || {
            let mut last = String::new();
            let res = ocr::download_models(&mut |s| {
                last = s.to_string();
                let _ = tx.send(BackgroundMsg::OcrProgress(s.to_string()));
            })
            .map_err(|e| e.to_string());
            let _ = tx.send(BackgroundMsg::OcrModelsDone(res));
        });
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("Open").clicked() {
                self.open_dialog();
            }
            if ui
                .add_enabled(self.session.is_some(), egui::Button::new("Save"))
                .clicked()
            {
                self.save();
            }
            if ui
                .add_enabled(self.session.is_some(), egui::Button::new("Save As"))
                .clicked()
            {
                self.save_as();
            }
            ui.separator();
            let count = self.page_count();
            if ui
                .add_enabled(self.page > 0, egui::Button::new("◀"))
                .clicked()
            {
                self.go_to_page(self.page.saturating_sub(1));
            }
            ui.label(format!("{} / {}", if count == 0 { 0 } else { self.page + 1 }, count));
            if ui
                .add_enabled(self.page + 1 < count, egui::Button::new("▶"))
                .clicked()
            {
                self.go_to_page(self.page + 1);
            }
            ui.separator();
            if ui
                .selectable_label(self.fit_width, "Fit width")
                .clicked()
            {
                self.fit_width = !self.fit_width;
                self.invalidate_textures();
            }
            if ui.button("−").clicked() {
                self.fit_width = false;
                self.zoom = (self.zoom / 1.15).max(0.25);
                self.invalidate_textures();
            }
            ui.label(format!("{:.0}%", self.zoom * 100.0));
            if ui.button("+").clicked() {
                self.fit_width = false;
                self.zoom = (self.zoom * 1.15).min(4.0);
                self.invalidate_textures();
            }
            ui.separator();
            ui.label("Search");
            let resp = ui.text_edit_singleline(&mut self.search_query);
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.run_search();
            }
            if ui.button("Find").clicked() {
                self.run_search();
            }
        });

        ui.horizontal_wrapped(|ui| {
            ui.label("Edit:");
            if ui
                .add_enabled(self.session.is_some(), egui::Button::new("Delete page"))
                .clicked()
            {
                if let Some(s) = self.session.as_mut() {
                    match s.delete_page(self.page) {
                        Ok(()) => {
                            self.page = self.page.min(s.page_count().unwrap_or(1).saturating_sub(1));
                            self.invalidate_textures();
                            self.status = "Page deleted".into();
                        }
                        Err(e) => self.error = Some(e.to_string()),
                    }
                }
            }
            if ui
                .add_enabled(self.session.is_some(), egui::Button::new("Rotate ⟳"))
                .clicked()
            {
                if let Some(s) = self.session.as_mut() {
                    if let Err(e) = s.rotate_page(self.page, 90) {
                        self.error = Some(e.to_string());
                    } else {
                        self.invalidate_textures();
                        self.status = "Rotated".into();
                    }
                }
            }
            if ui
                .add_enabled(
                    self.session.is_some() && self.page > 0,
                    egui::Button::new("Move up"),
                )
                .clicked()
            {
                if let Some(s) = self.session.as_mut() {
                    let from = self.page;
                    if let Err(e) = s.move_page(from, from - 1) {
                        self.error = Some(e.to_string());
                    } else {
                        self.invalidate_textures();
                        self.go_to_page(from - 1);
                    }
                }
            }
            if ui
                .add_enabled(
                    self.session.is_some() && self.page + 1 < self.page_count(),
                    egui::Button::new("Move down"),
                )
                .clicked()
            {
                if let Some(s) = self.session.as_mut() {
                    let from = self.page;
                    if let Err(e) = s.move_page(from, from + 1) {
                        self.error = Some(e.to_string());
                    } else {
                        self.invalidate_textures();
                        self.go_to_page(from + 1);
                    }
                }
            }
            ui.separator();
            ui.selectable_value(&mut self.mode, ToolMode::View, "View");
            ui.selectable_value(&mut self.mode, ToolMode::Crop, "Crop");
            ui.selectable_value(&mut self.mode, ToolMode::VisualSign, "Sign");
            ui.selectable_value(&mut self.mode, ToolMode::CertSign, "Cert sign");
            if ui.button("Compress…").clicked() {
                self.show_compress = true;
            }
            if ui.button("OCR…").clicked() {
                self.show_ocr = true;
            }
        });
    }

    fn run_search(&mut self) {
        self.search_hits.clear();
        let Some(session) = self.session.as_ref() else {
            return;
        };
        match session.search(self.page, &self.search_query) {
            Ok(hits) => {
                self.status = format!("{} hits on this page", hits.len());
                self.search_hits = hits;
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    fn side_panels(&mut self, ui: &mut egui::Ui) {
        if self.mode == ToolMode::VisualSign {
            egui::Panel::right("sign_panel")
                .default_size(320.0)
                .show(ui, |ui| {
                    ui.heading("Visual signature");
                    ui.label("Draw below, then click Place on page.");
                    let (resp, painter) = ui.allocate_painter(
                        egui::vec2(self.sig_pad.pad_w as f32, self.sig_pad.pad_h as f32),
                        egui::Sense::click_and_drag(),
                    );
                    painter.rect_filled(resp.rect, 4.0, egui::Color32::from_gray(245));
                    painter.rect_stroke(
                        resp.rect,
                        4.0,
                        egui::Stroke::new(1.0, egui::Color32::GRAY),
                        egui::StrokeKind::Outside,
                    );
                    if let Some(pos) = resp.interact_pointer_pos() {
                        let local = pos - resp.rect.min;
                        if resp.drag_started() {
                            self.sig_pad.begin(local.x, local.y);
                        } else if resp.dragged() {
                            self.sig_pad.drag(local.x, local.y);
                        } else if resp.drag_stopped() {
                            self.sig_pad.end();
                        }
                    }
                    for stroke in &self.sig_pad.strokes {
                        for pair in stroke.windows(2) {
                            painter.line_segment(
                                [
                                    resp.rect.min + egui::vec2(pair[0].0, pair[0].1),
                                    resp.rect.min + egui::vec2(pair[1].0, pair[1].1),
                                ],
                                egui::Stroke::new(2.0, egui::Color32::from_rgb(20, 20, 40)),
                            );
                        }
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Clear").clicked() {
                            self.sig_pad.clear();
                            self.imported_sig = None;
                        }
                        if ui.button("Import PNG…").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Image", &["png", "jpg", "jpeg"])
                                .pick_file()
                            {
                                match image::open(&path) {
                                    Ok(img) => {
                                        let rgba = img.to_rgba8();
                                        self.imported_sig =
                                            Some((rgba.width(), rgba.height(), rgba.into_raw()));
                                        self.status = "Signature image loaded".into();
                                    }
                                    Err(e) => self.error = Some(e.to_string()),
                                }
                            }
                        }
                        if ui
                            .add_enabled(
                                !self.sig_pad.is_empty() || self.imported_sig.is_some(),
                                egui::Button::new("Place on page"),
                            )
                            .clicked()
                        {
                            self.placing_sig = true;
                            self.status = "Click on the page to place signature".into();
                        }
                    });
                });
        }

        if self.mode == ToolMode::CertSign {
            egui::Panel::right("cert_panel")
                .default_size(320.0)
                .show(ui, |ui| {
                    ui.heading("Certificate signature");
                    ui.label("Import a .p12 / .pfx file.");
                    if ui.button("Choose PKCS#12…").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("PKCS#12", &["p12", "pfx"])
                            .pick_file()
                        {
                            self.cert_path = Some(path);
                        }
                    }
                    if let Some(p) = &self.cert_path {
                        ui.label(p.display().to_string());
                    }
                    ui.horizontal(|ui| {
                        ui.label("Password");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.cert_password).password(true),
                        );
                    });
                    if ui.button("Load certificate").clicked() {
                        if let Some(path) = self.cert_path.clone() {
                            match CertIdentity::from_pkcs12(&path, &self.cert_password) {
                                Ok(id) => {
                                    self.status = format!("Loaded: {}", id.subject);
                                    self.cert_identity = Some(id);
                                }
                                Err(e) => self.error = Some(e.to_string()),
                            }
                        }
                    }
                    if let Some(id) = &self.cert_identity {
                        ui.label(format!("Subject: {}", id.subject));
                    }
                    ui.label("Reason");
                    ui.text_edit_singleline(&mut self.cert_reason);
                    if ui
                        .add_enabled(
                            self.cert_identity.is_some() && self.session.is_some(),
                            egui::Button::new("Place & sign"),
                        )
                        .clicked()
                    {
                        self.placing_cert = true;
                        self.cert_place = None;
                        self.status = "Drag a rectangle on the page for the signature".into();
                    }
                });
        }
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        if self.show_compress {
            egui::Window::new("Compress PDF")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Choose a compression preset and save a new file.");
                    for preset in [
                        CompressPreset::Fast,
                        CompressPreset::Balanced,
                        CompressPreset::Small,
                    ] {
                        ui.radio_value(&mut self.compress_preset, preset, preset.label());
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_compress = false;
                        }
                        if ui
                            .add_enabled(self.session.is_some(), egui::Button::new("Compress & Save"))
                            .clicked()
                        {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("PDF", &["pdf"])
                                .set_file_name("compressed.pdf")
                                .save_file()
                            {
                                if let Some(s) = self.session.as_mut() {
                                    match s.compress_save(&path, self.compress_preset) {
                                        Ok(()) => {
                                            self.status =
                                                format!("Compressed → {}", path.display());
                                            self.show_compress = false;
                                        }
                                        Err(e) => self.error = Some(e.to_string()),
                                    }
                                }
                            }
                        }
                    });
                });
        }

        if self.show_ocr {
            egui::Window::new("OCR")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Local OCR uses downloadable neural models (Latin script).");
                    if ocr::models_installed() {
                        ui.colored_label(egui::Color32::DARK_GREEN, "Models installed");
                    } else {
                        ui.colored_label(egui::Color32::DARK_RED, "Models not installed");
                    }
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(!self.busy, egui::Button::new("Download models"))
                            .clicked()
                        {
                            self.download_ocr_models();
                        }
                        if ui
                            .add_enabled(
                                !self.busy && self.session.is_some() && ocr::models_installed(),
                                egui::Button::new("OCR this page"),
                            )
                            .clicked()
                        {
                            self.start_ocr_page();
                        }
                        if ui.button("Close").clicked() {
                            self.show_ocr = false;
                        }
                    });
                    if self.busy {
                        ui.spinner();
                        ui.label(&self.status);
                    }
                });
        }

        if let Some(err) = self.error.clone() {
            egui::Window::new("Error")
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(&err);
                    if ui.button("OK").clicked() {
                        self.error = None;
                    }
                });
        }
    }

    fn viewer(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.session.is_none() {
            ui.centered_and_justified(|ui| {
                ui.label("Open a PDF file to view it.");
            });
            return;
        }

        let avail_w = ui.available_width() - 24.0;
        self.update_zoom(avail_w.max(100.0));

        let count = self.page_count();
        let zoom = self.zoom;
        let mode = self.mode;
        let viewport = ui.clip_rect();

        // Warm textures around the current page (updates as you scroll).
        let near_start = self.page.saturating_sub(TEXTURE_CACHE_RADIUS as usize);
        let near_end = (self.page + TEXTURE_CACHE_RADIUS as usize + 1).min(count);
        for page_idx in near_start..near_end {
            self.ensure_page_texture(ctx, page_idx);
        }
        self.prune_texture_cache();

        let scroll_target = self.scroll_to_page.take();
        let mut best_page = self.page;
        let mut best_overlap = 0.0f32;
        let mut interaction: Option<PageInteraction> = None;

        egui::ScrollArea::vertical()
            .id_salt("pdf_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                for page_idx in 0..count {
                    let size = self.page_display_size(page_idx);

                    ui.horizontal(|ui| {
                        let pad = ((ui.available_width() - size.x) * 0.5).max(0.0);
                        ui.add_space(pad);

                        let sense = if matches!(
                            mode,
                            ToolMode::Crop | ToolMode::VisualSign | ToolMode::CertSign
                        ) {
                            egui::Sense::click_and_drag()
                        } else {
                            egui::Sense::hover()
                        };

                        let (response, painter) = ui.allocate_painter(size, sense);
                        let rect = response.rect;

                        if scroll_target == Some(page_idx) {
                            response.scroll_to_me(Some(egui::Align::TOP));
                        }

                        let overlap = rect.intersect(viewport).height();
                        if overlap > best_overlap {
                            best_overlap = overlap;
                            best_page = page_idx;
                        }

                        if let Some(tex) = self.page_textures.get(&page_idx) {
                            painter.image(
                                tex.id(),
                                rect,
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                egui::Color32::WHITE,
                            );
                        } else {
                            painter.rect_filled(rect, 0.0, egui::Color32::from_gray(230));
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("Page {}", page_idx + 1),
                                egui::FontId::proportional(16.0),
                                egui::Color32::DARK_GRAY,
                            );
                        }

                        painter.rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(1.0, egui::Color32::from_gray(180)),
                            egui::StrokeKind::Outside,
                        );

                        if page_idx == self.page {
                            for hit in &self.search_hits {
                                let r = pdf_rect_to_screen(hit, zoom, rect);
                                painter.rect_filled(
                                    r,
                                    0.0,
                                    egui::Color32::from_rgba_unmultiplied(255, 220, 0, 80),
                                );
                            }
                        }

                        // Live drag tracking + deferred commit events.
                        match mode {
                            ToolMode::Crop => {
                                if response.drag_started() {
                                    if let Some(pos) = response.interact_pointer_pos() {
                                        self.crop_start = Some(pos);
                                        self.crop_end = Some(pos);
                                        self.crop_page = Some(page_idx);
                                    }
                                }
                                if self.crop_page == Some(page_idx) {
                                    if response.dragged() {
                                        self.crop_end = response.interact_pointer_pos();
                                    }
                                    if response.drag_stopped() {
                                        if let (Some(a), Some(b)) =
                                            (self.crop_start, self.crop_end)
                                        {
                                            interaction = Some(PageInteraction::CropEnd {
                                                page: page_idx,
                                                start: a,
                                                end: b,
                                                rect,
                                            });
                                        }
                                    }
                                    if let (Some(a), Some(b)) = (self.crop_start, self.crop_end) {
                                        painter.rect_stroke(
                                            egui::Rect::from_two_pos(a, b),
                                            0.0,
                                            egui::Stroke::new(
                                                1.5,
                                                egui::Color32::from_rgb(0, 120, 255),
                                            ),
                                            egui::StrokeKind::Outside,
                                        );
                                    }
                                }
                            }
                            ToolMode::VisualSign if self.placing_sig => {
                                if response.clicked() {
                                    if let Some(pos) = response.interact_pointer_pos() {
                                        interaction = Some(PageInteraction::SignClick {
                                            page: page_idx,
                                            pos,
                                            rect,
                                        });
                                    }
                                }
                            }
                            ToolMode::CertSign if self.placing_cert => {
                                if response.drag_started() {
                                    if let Some(pos) = response.interact_pointer_pos() {
                                        self.cert_place = Some((pos, pos));
                                        self.cert_page = Some(page_idx);
                                    }
                                }
                                if self.cert_page == Some(page_idx) {
                                    if response.dragged() {
                                        if let Some((start, _)) = self.cert_place {
                                            if let Some(p) = response.interact_pointer_pos() {
                                                self.cert_place = Some((start, p));
                                            }
                                        }
                                    }
                                    if response.drag_stopped() {
                                        if let Some((a, b)) = self.cert_place {
                                            interaction = Some(PageInteraction::CertEnd {
                                                page: page_idx,
                                                start: a,
                                                end: b,
                                                rect,
                                            });
                                        }
                                    }
                                    if let Some((a, b)) = self.cert_place {
                                        painter.rect_stroke(
                                            egui::Rect::from_two_pos(a, b),
                                            0.0,
                                            egui::Stroke::new(
                                                1.5,
                                                egui::Color32::from_rgb(0, 160, 80),
                                            ),
                                            egui::StrokeKind::Outside,
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                    });

                    ui.add_space(PAGE_GAP);
                }
            });

        if best_overlap > 0.0 && best_page != self.page {
            self.page = best_page;
        }

        if let Some(ev) = interaction {
            self.apply_page_interaction(ev, zoom);
        }
    }

    fn apply_page_interaction(&mut self, ev: PageInteraction, zoom: f32) {
        match ev {
            PageInteraction::CropEnd {
                page,
                start,
                end,
                rect,
            } => {
                if let Some(session) = self.session.as_mut() {
                    if let Ok((_, page_h)) = session.page_size(page) {
                        let screen = egui::Rect::from_two_pos(start, end);
                        let crop = screen_rect_to_pdf(screen, zoom, rect, page_h);
                        if let Err(e) = session.set_crop(page, crop) {
                            self.error = Some(e.to_string());
                        } else {
                            self.page = page;
                            self.status = format!("Crop applied on page {}", page + 1);
                            self.invalidate_textures();
                        }
                    }
                }
                self.crop_start = None;
                self.crop_end = None;
                self.crop_page = None;
            }
            PageInteraction::SignClick { page, pos, rect } => {
                self.page = page;
                self.place_visual_signature(pos, rect, zoom);
                self.placing_sig = false;
            }
            PageInteraction::CertEnd {
                page,
                start,
                end,
                rect,
            } => {
                self.page = page;
                self.apply_cert_signature(start, end, rect, zoom);
                self.placing_cert = false;
                self.cert_place = None;
                self.cert_page = None;
            }
        }
    }

    fn place_visual_signature(&mut self, pos: egui::Pos2, page_rect: egui::Rect, zoom: f32) {
        let (w, h, rgba) = if let Some((w, h, ref rgba)) = self.imported_sig {
            (w, h, rgba.clone())
        } else {
            self.sig_pad.to_rgba()
        };
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let Ok((_, page_h)) = session.page_size(self.page) else {
            return;
        };
        let sig_w_pt = (w as f32 / zoom) * 0.5; // display half pad size-ish
        let sig_h_pt = (h as f32 / zoom) * 0.5;
        let local = pos - page_rect.min;
        let pdf_x = local.x / zoom;
        let pdf_y_top = local.y / zoom;
        let pdf_y = page_h - pdf_y_top;
        let rect = PdfRect {
            x0: pdf_x,
            y0: pdf_y - sig_h_pt,
            x1: pdf_x + sig_w_pt,
            y1: pdf_y,
        };
        match session.stamp_image(self.page, &rgba, w, h, rect) {
            Ok(()) => {
                self.status = "Signature stamped".into();
                self.invalidate_textures();
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    fn apply_cert_signature(
        &mut self,
        a: egui::Pos2,
        b: egui::Pos2,
        page_rect: egui::Rect,
        zoom: f32,
    ) {
        let Some(identity) = self.cert_identity.as_ref() else {
            return;
        };
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let Ok((_, page_h)) = session.page_size(self.page) else {
            return;
        };
        let screen = egui::Rect::from_two_pos(a, b);
        let crop = screen_rect_to_pdf(screen, zoom, page_rect, page_h);
        let rect = [crop.x0, crop.y0, crop.x1, crop.y1];

        let bytes = match session.write_bytes(CompressPreset::Balanced.write_options()) {
            Ok(b) => b,
            Err(e) => {
                self.error = Some(e.to_string());
                return;
            }
        };
        let reason = self.cert_reason.clone();
        let page = self.page;
        match cert::sign_pdf_bytes(&bytes, identity, page, rect, &reason) {
            Ok(signed) => {
                // Write to temp then reload, or save-as.
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .set_file_name("signed.pdf")
                    .save_file()
                {
                    if let Err(e) = std::fs::write(&path, &signed) {
                        self.error = Some(e.to_string());
                        return;
                    }
                    match DocumentSession::open(&path) {
                        Ok(s) => {
                            self.session = Some(s);
                            self.invalidate_textures();
                            self.status = format!("Signed → {}", path.display());
                        }
                        Err(e) => {
                            // File written even if reopen fails
                            self.status = format!("Signed file written ({e})");
                        }
                    }
                }
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        let next = ctx.input(|i| i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::PageDown));
        let prev = ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::PageUp));
        if next && self.page + 1 < self.page_count() {
            self.go_to_page(self.page + 1);
        }
        if prev && self.page > 0 {
            self.go_to_page(self.page.saturating_sub(1));
        }
        let open = ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O));
        let save = ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S));
        if open {
            self.open_dialog();
        }
        if save {
            if ctx.input(|i| i.modifiers.shift) {
                self.save_as();
            } else {
                self.save();
            }
        }
    }
}

impl eframe::App for PdfApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_background();
        self.handle_keys(ctx);
        self.dialogs(ctx);
        if self.busy {
            ctx.request_repaint();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        egui::Panel::top("toolbar").show(ui, |ui| {
            self.toolbar(ui);
            if let Some(s) = &self.session {
                if s.dirty {
                    ui.colored_label(egui::Color32::from_rgb(180, 100, 0), "Modified");
                }
            }
            ui.label(&self.status);
        });

        self.side_panels(ui);

        egui::CentralPanel::default().show(ui, |ui| {
            self.viewer(ui, &ctx);
        });
    }
}

fn pdf_rect_to_screen(r: &PdfRect, zoom: f32, page_rect: egui::Rect) -> egui::Rect {
    // PDF y grows up; screen y grows down. Approximate using page_rect height.
    let page_h = page_rect.height() / zoom;
    let x0 = page_rect.min.x + r.x0 * zoom;
    let x1 = page_rect.min.x + r.x1 * zoom;
    let y0 = page_rect.min.y + (page_h - r.y1) * zoom;
    let y1 = page_rect.min.y + (page_h - r.y0) * zoom;
    egui::Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(x1, y1))
}

fn screen_rect_to_pdf(
    screen: egui::Rect,
    zoom: f32,
    page_rect: egui::Rect,
    page_h: f32,
) -> PdfRect {
    let local = egui::Rect::from_min_max(
        egui::pos2(
            (screen.min.x - page_rect.min.x) / zoom,
            (screen.min.y - page_rect.min.y) / zoom,
        ),
        egui::pos2(
            (screen.max.x - page_rect.min.x) / zoom,
            (screen.max.y - page_rect.min.y) / zoom,
        ),
    );
    PdfRect {
        x0: local.min.x.min(local.max.x),
        x1: local.min.x.max(local.max.x),
        y0: page_h - local.max.y.max(local.min.y),
        y1: page_h - local.min.y.min(local.max.y),
    }
}

#[allow(dead_code)]
fn _unused_error_check(_: AppError) {}
