use crate::batch::{Msg, run_batch};
use crate::files::collect_images;
use crate::settings::{AvailableEncoders, OutputFormat, Quality, Resolution};
use crate::theme::{Theme, ThemeColors};
use crate::ui::{
    banner, caption, card, danger_button, field_label, metric, pill, primary_button, quiet_button,
    section_title,
};
use eframe::egui;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;

const LOG_CAP: usize = 500;
const CONTENT_MAX_WIDTH: f32 = 720.0;

pub struct App {
    folder: String,
    last_scanned: String,
    file_count: usize,
    resolution: Resolution,
    quality: Quality,
    format: OutputFormat,
    keep_exif: bool,
    converting: bool,
    cancel: Arc<AtomicBool>,
    log: VecDeque<LogLine>,
    rx: Option<mpsc::Receiver<Msg>>,
    progress: (usize, usize),
    last_out_dir: Option<PathBuf>,
    theme: Theme,
    available_encoders: AvailableEncoders,
}

struct LogLine {
    is_err: bool,
    text: String,
}

impl Default for App {
    fn default() -> Self {
        let available_encoders = AvailableEncoders::detect();
        let mut app = Self {
            folder: String::new(),
            last_scanned: String::new(),
            file_count: 0,
            resolution: Resolution::High,
            quality: Quality::Medium,
            format: available_encoders
                .first_available()
                .unwrap_or(OutputFormat::Jpg),
            keep_exif: true,
            converting: false,
            cancel: Arc::new(AtomicBool::new(false)),
            log: VecDeque::new(),
            rx: None,
            progress: (0, 0),
            last_out_dir: None,
            theme: Theme::Light,
            available_encoders,
        };
        app.select_initial_folder();
        app
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_messages();
        self.handle_dropped_files(ctx);
        if self.converting {
            ctx.request_repaint();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let colors = ThemeColors::for_theme(self.theme);
        colors.apply(ui.ctx());

        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(ui.ctx().global_style().as_ref())
                    .fill(colors.bg)
                    .inner_margin(egui::Margin::symmetric(24, 22)),
            )
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let pad = ((ui.available_width() - CONTENT_MAX_WIDTH) / 2.0).max(0.0);
                        ui.horizontal(|ui| {
                            ui.add_space(pad);
                            ui.vertical(|ui| {
                                ui.set_width(ui.available_width().min(CONTENT_MAX_WIDTH));
                                self.body(ui, &colors);
                            });
                            ui.add_space(pad);
                        });
                    });
            });
    }
}

impl App {
    fn select_initial_folder(&mut self) {
        let Ok(cwd) = std::env::current_dir() else {
            return;
        };
        self.folder = cwd.display().to_string();
        self.last_scanned = self.folder.clone();
        self.file_count = collect_images(&cwd).len();
    }

    fn body(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        self.header(ui, colors);
        ui.add_space(20.0);

        if !self.available_encoders.any() {
            banner(ui, colors, colors.danger, |ui| {
                ui.label(
                    egui::RichText::new(
                        "No encoders found on PATH. Install at least one of cjpeg, avifenc, cjxl, or heif-enc to enable conversion.",
                    )
                    .size(12.0)
                    .strong()
                    .color(colors.danger),
                );
            });
            ui.add_space(14.0);
        }

        card(ui, colors, |ui| self.source_section(ui, colors));
        ui.add_space(14.0);
        card(ui, colors, |ui| self.output_section(ui, colors));
        ui.add_space(14.0);
        card(ui, colors, |ui| self.action_section(ui, colors));

        if !self.log.is_empty() {
            ui.add_space(14.0);
            card(ui, colors, |ui| self.log_section(ui, colors));
        }
    }

    fn header(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("img-convert")
                        .size(30.0)
                        .strong()
                        .color(colors.text),
                );
                ui.label(
                    egui::RichText::new("Batch image resizing and compression")
                        .size(13.0)
                        .color(colors.muted),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if quiet_button(ui, colors, format!("{} theme", self.theme.other_label()))
                    .on_hover_text("Switch theme")
                    .clicked()
                {
                    self.theme = self.theme.toggled();
                }
            });
        });
    }

    fn source_section(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        section_title(ui, colors, "Source");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            let response = ui.add_enabled(
                !self.converting,
                egui::TextEdit::singleline(&mut self.folder)
                    .hint_text("Folder containing images — drag a folder here")
                    .desired_width(ui.available_width() - 96.0),
            );
            if response.lost_focus() {
                self.update_file_count();
            }
            ui.add_enabled_ui(!self.converting, |ui| {
                if quiet_button(ui, colors, "Browse").clicked()
                    && let Some(folder) = rfd::FileDialog::new().pick_folder()
                {
                    self.folder = folder.display().to_string();
                    self.update_file_count();
                }
            });
        });

        ui.add_space(14.0);

        let path = PathBuf::from(&self.folder);
        let folder_label = if path.is_dir() {
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("selected folder")
                .to_owned()
        } else {
            "no valid folder selected".to_owned()
        };

        if self.file_count > 0 {
            let noun = if self.file_count == 1 {
                "image"
            } else {
                "images"
            };
            metric(
                ui,
                colors,
                &self.file_count.to_string(),
                &format!("{noun} in {folder_label}"),
                true,
            );
        } else if path.is_dir() {
            caption(ui, colors, format!("No images in {folder_label}"));
        } else {
            caption(ui, colors, "Drag a folder here or pick one with Browse");
        }
    }

    fn output_section(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        section_title(ui, colors, "Output");
        ui.add_space(12.0);

        field_label(ui, colors, "Format");
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            for format in OutputFormat::ALL {
                let enabled = self.available_encoders.has(format) && !self.converting;
                let hover = if self.available_encoders.has(format) {
                    format!("encoder: {}", format.encoder())
                } else {
                    format!("missing encoder: {}", format.encoder())
                };
                pill(
                    ui,
                    colors,
                    &mut self.format,
                    format,
                    format.label(),
                    enabled,
                )
                .on_hover_text(hover);
            }
        });

        ui.add_space(18.0);
        ui.separator();
        ui.add_space(14.0);

        ui.horizontal_top(|ui| {
            let half = (ui.available_width() - 18.0) / 2.0;
            ui.vertical(|ui| {
                ui.set_width(half);
                field_label(ui, colors, "Resolution");
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    for resolution in Resolution::ALL {
                        pill(
                            ui,
                            colors,
                            &mut self.resolution,
                            resolution,
                            resolution.short_label(),
                            !self.converting,
                        );
                    }
                });
                ui.add_space(6.0);
                let resolution_caption = match self.resolution.short_side() {
                    Some(_) => format!("Short side: {}", self.resolution.label()),
                    None => "No resizing".to_owned(),
                };
                caption(ui, colors, resolution_caption);
            });
            ui.add_space(18.0);
            ui.vertical(|ui| {
                ui.set_width(half);
                field_label(ui, colors, "Quality");
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    for quality in Quality::ALL {
                        pill(
                            ui,
                            colors,
                            &mut self.quality,
                            quality,
                            quality.label(),
                            !self.converting,
                        );
                    }
                });
                ui.add_space(6.0);
                caption(ui, colors, self.quality.label_with_value(self.format));
            });
        });

        ui.add_space(16.0);
        ui.add_enabled_ui(!self.converting, |ui| {
            ui.checkbox(&mut self.keep_exif, "Preserve EXIF metadata");
        });
    }

    fn action_section(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        let can_convert =
            self.file_count > 0 && !self.converting && self.available_encoders.has(self.format);
        let label = if self.converting {
            "Cancel".to_owned()
        } else if can_convert {
            format!("Convert {} to {}", self.file_count, self.format.label())
        } else if !self.available_encoders.has(self.format) {
            format!("{} encoder unavailable", self.format.label())
        } else {
            "Choose a folder with images".to_owned()
        };

        let button = if self.converting {
            danger_button(colors, &label, [ui.available_width(), 52.0])
        } else {
            primary_button(colors, &label, [ui.available_width(), 52.0])
        };

        if ui
            .add_enabled(can_convert || self.converting, button)
            .clicked()
        {
            if self.converting {
                self.cancel.store(true, Ordering::Relaxed);
            } else {
                self.start_conversion();
            }
        }

        if self.progress.1 > 0 {
            ui.add_space(12.0);
            let progress = self.progress.0 as f32 / self.progress.1 as f32;
            ui.add(
                egui::ProgressBar::new(progress)
                    .desired_width(ui.available_width())
                    .fill(colors.accent)
                    .text(format!(
                        "{:.0}%   ·   {} of {}",
                        progress * 100.0,
                        self.progress.0,
                        self.progress.1
                    )),
            );
        }

        if !self.converting && self.last_out_dir.is_some() {
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if quiet_button(ui, colors, "Open output folder").clicked()
                    && let Some(dir) = &self.last_out_dir
                {
                    open_folder(dir);
                }
                if quiet_button(ui, colors, "Clear log").clicked() {
                    self.log.clear();
                    self.progress = (0, 0);
                    self.last_out_dir = None;
                }
            });
        }
    }

    fn log_section(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        section_title(ui, colors, "Activity");
        ui.add_space(8.0);

        egui::ScrollArea::vertical()
            .max_height(180.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for line in &self.log {
                    let color = if line.is_err {
                        colors.danger
                    } else {
                        colors.muted
                    };
                    ui.label(
                        egui::RichText::new(&line.text)
                            .size(11.0)
                            .monospace()
                            .color(color),
                    );
                }
                ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
            });
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        if self.converting {
            return;
        }
        let dropped = ctx.input(|input| input.raw.dropped_files.clone());
        let Some(file) = dropped.into_iter().next() else {
            return;
        };
        let Some(path) = file.path else {
            return;
        };
        let folder = if path.is_dir() {
            Some(path)
        } else {
            path.parent().map(Path::to_path_buf)
        };
        if let Some(folder) = folder {
            self.folder = folder.display().to_string();
            self.update_file_count();
        }
    }

    fn update_file_count(&mut self) {
        if self.folder == self.last_scanned {
            return;
        }
        self.last_scanned.clone_from(&self.folder);
        let path = PathBuf::from(&self.folder);
        self.file_count = if path.is_dir() {
            collect_images(&path).len()
        } else {
            0
        };
    }

    fn poll_messages(&mut self) {
        let Some(rx) = &self.rx else {
            return;
        };

        let mut messages = Vec::new();
        while let Ok(message) = rx.try_recv() {
            messages.push(message);
        }

        for message in messages {
            match message {
                Msg::Started(total) => {
                    self.progress = (0, total);
                    self.push_log(false, format!("Converting {total} images..."));
                }
                Msg::FileOk(src, dst) => {
                    self.progress.0 += 1;
                    self.push_log(false, format!("OK  {src} -> {dst}"));
                }
                Msg::FileErr(src, err) => {
                    self.progress.0 += 1;
                    self.push_log(true, format!("ERR {src}: {err}"));
                }
                Msg::Done {
                    ok,
                    fail,
                    skipped,
                    cancelled,
                    out_dir,
                } => {
                    let summary = if cancelled {
                        format!("Cancelled: {ok} done, {fail} failed, {skipped} skipped")
                    } else {
                        format!("Done: {ok} succeeded, {fail} failed")
                    };
                    self.push_log(cancelled || fail > 0, summary);
                    self.converting = false;
                    self.last_out_dir = Some(out_dir);
                }
            }
        }
    }

    fn start_conversion(&mut self) {
        self.converting = true;
        self.log.clear();
        self.progress = (0, 0);
        self.last_out_dir = None;
        self.cancel = Arc::new(AtomicBool::new(false));

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        let folder = PathBuf::from(&self.folder);
        let short_side = self.resolution.short_side();
        let quality = self.quality.value_for(self.format);
        let format = self.format;
        let keep_exif = self.keep_exif;
        let cancel = self.cancel.clone();

        thread::spawn(move || {
            run_batch(folder, short_side, quality, format, keep_exif, cancel, tx)
        });
    }

    fn push_log(&mut self, is_err: bool, text: String) {
        if self.log.len() >= LOG_CAP {
            self.log.pop_front();
        }
        self.log.push_back(LogLine { is_err, text });
    }
}

fn open_folder(path: &Path) {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer").arg(path).spawn().or_else(|_| {
            Command::new("cmd")
                .args(["/C", "start", "", ""])
                .arg(path)
                .spawn()
        });
    }
    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg(path).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let _ = Command::new("xdg-open").arg(path).spawn();
}
