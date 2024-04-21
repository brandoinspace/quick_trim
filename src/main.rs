#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env,
    process::{Command, Stdio},
};

use eframe::egui::{self, Color32};

// TODO:
// - make async
// - scroll with images to select time instead of inputting manually
// - windows right click open with
// - millisecond percision
// - settings window
// - scrubbers on same y
fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon.png")[..]).unwrap())
            .with_inner_size([520.0, 440.0])
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native("Quick Trim", options, Box::new(|_cc| Box::<QuickTrim>::default()))
}

// File picker based off of:
// https://github.com/emilk/egui/blob/master/examples/file_dialog/src/main.rs
#[derive(PartialEq)]
struct QuickTrim {
    picked_path: Option<String>,
    start_trim: f32,
    end_trim: f32,
    video_length: u32,
    output_name: String,
    output_location: Option<String>,
    show_no_file_error: bool,
    show_no_name_error: bool,
    trim_can_continue: bool,
    trim_finished: bool,
    trim_to_end: bool,
    overwrite: bool,
    slow_trim: bool,
    scrubber_is_visible: bool,
    ffmpeg_gen_output_made: bool,
    ffmpeg_gen_output: Option<String>,
    opened_using_open_with_windows: bool,
    args: Option<Vec<String>>,
}

impl Default for QuickTrim {
    fn default() -> Self {
        Self {
            picked_path: None,
            start_trim: 0.0,
            end_trim: 0.0,
            video_length: 0,
            output_name: "output.mp4".to_owned(),
            output_location: None,
            show_no_file_error: false,
            show_no_name_error: false,
            trim_can_continue: false,
            trim_finished: false,
            trim_to_end: false,
            overwrite: true,
            slow_trim: false,
            scrubber_is_visible: false,
            ffmpeg_gen_output_made: false,
            ffmpeg_gen_output: None,
            opened_using_open_with_windows: false,
            args: None,
        }
    }
}

impl eframe::App for QuickTrim {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(Color32::WHITE);
            ui.visuals_mut().panel_fill = Color32::from_hex("#353535").unwrap();

            // let temp_dir = TempDir::new().unwrap();
            // let frame_temp = temp_dir.child("frame1");
            ui.vertical_centered_justified(|ui| {
                ui.heading("Quick Trim");

                ui.add_space(15.0);

                if !self.opened_using_open_with_windows {
                    let args: Vec<String> = env::args().collect();
                    if !args.is_empty() {
                        self.args = Some(args);
                    }
                    self.opened_using_open_with_windows = true;
                }
                if let Some(argss) = &self.args {
                    if argss.len() > 1 {
                        self.picked_path = Some(argss[1].clone());
                    }
                }

                egui::Grid::new("Options")
                    .num_columns(2)
                    .spacing([20.0, 10.0])
                    .min_col_width(ui.available_width() / 4.0)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("File");
                        ui.horizontal(|ui| {
                            if ui.button("Open file...").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_title("Open File to Trim")
                                    .add_filter("Video File", &["mp4"])
                                    .pick_file()
                                {
                                    self.picked_path = Some(path.display().to_string());
                                    let cmd = Command::new("ffprobe")
                                        .args([
                                            "-v",
                                            "error",
                                            "-select_streams",
                                            "v:0",
                                            "-show_entries",
                                            "stream=duration",
                                            "-of",
                                            "default=noprint_wrappers=1:nokey=1",
                                            &self.picked_path.as_ref().unwrap(),
                                        ])
                                        .stderr(Stdio::piped())
                                        .output()
                                        .expect("Could not get video length!");
                                    self.end_trim = String::from_utf8_lossy(&cmd.stdout).into_owned().trim_end().parse::<f32>().unwrap();
                                    self.start_trim = 0.0;
                                    self.video_length = self.end_trim as u32;
                                    self.scrubber_is_visible = true;
                                }
                            }
                            if let Some(picked_path) = &self.picked_path {
                                ui.label(format!("({picked_path})"));
                                // let args = [
                                //     "-i",
                                //     self.picked_path.as_ref().unwrap(),
                                //     "-ss",
                                //     "00:00:00",
                                //     "-s",
                                //     "650x390",
                                //     "-vframes",
                                //     "1",
                                //     "-c:v",
                                //     "png",
                                //     "-f",
                                //     "image2pipe",
                                //     "pipe:1",
                                // ];
                                // let f = Command::new("ffmpeg")
                                //     .args(args)
                                //     .output()
                                //     .expect("Could not get image frame!");
                                // std::fs::write(&)
                                // cmd_frame = Some(f);
                                // let frame = cmd_frame.stdin.unwrap().re;
                            }
                        });
                        ui.end_row();

                        ui.label("Output");
                        ui.horizontal(|ui| {
                            if ui.button("Open folder...").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_title("Set Ouput")
                                    .add_filter("Video File", &["mp4"])
                                    .set_file_name(&self.output_name)
                                    .save_file()
                                {
                                    self.output_location = Some(path.display().to_string());
                                }
                            }
                            if let Some(path) = &self.output_location {
                                ui.label(format!("({path})"));
                            }
                        });
                        ui.end_row();

                        ui.label("Start Trim");
                        // From https://docs.rs/egui/latest/egui/widgets/struct.DragValue.html#method.custom_formatter
                        let end_trim_clone = self.end_trim;
                        ui.add(
                            egui::DragValue::new(&mut self.start_trim)
                                .clamp_range(0.0..=end_trim_clone)
                                .custom_formatter(|n, _| num_to_time(n as f32))
                                .custom_parser(|s| {
                                    let parts: Vec<&str> = s.split(':').collect();
                                    if parts.len() == 3 {
                                        parts[0]
                                            .parse::<i32>()
                                            .and_then(|h| {
                                                parts[1]
                                                    .parse::<i32>()
                                                    .and_then(|m| parts[2].parse::<i32>().map(|s| ((h * 60 * 60) + (m * 60) + s) as f64))
                                            })
                                            .ok()
                                    } else {
                                        None
                                    }
                                }),
                        );
                        ui.end_row();

                        ui.label("End Trim");
                        ui.horizontal(|ui| {
                            ui.add_enabled(
                                !self.trim_to_end,
                                egui::DragValue::new(&mut self.end_trim)
                                    .clamp_range(0.0..=end_trim_clone)
                                    .custom_formatter(|n, _| num_to_time(n as f32))
                                    .custom_parser(|s| {
                                        let parts: Vec<&str> = s.split(':').collect();
                                        if parts.len() == 3 {
                                            parts[0]
                                                .parse::<i32>()
                                                .and_then(|h| {
                                                    parts[1]
                                                        .parse::<i32>()
                                                        .and_then(|m| parts[2].parse::<i32>().map(|s| ((h * 60 * 60) + (m * 60) + s) as f64))
                                                })
                                                .ok()
                                        } else {
                                            None
                                        }
                                    }),
                            );
                            ui.checkbox(&mut self.trim_to_end, "To End")
                        });
                        ui.end_row();

                        ui.label("Extra");
                        ui.horizontal(|ui| {
                            // maybe just check if file exists at output path and if so, add this automatically?
                            ui.checkbox(&mut self.overwrite, "Overwrite Existing");
                            ui.checkbox(&mut self.slow_trim, "Slow Trim (Blocking)");
                        });
                        ui.end_row();
                    });
            });

            ui.add_space(20.0);

            ui.add_visible(
                self.scrubber_is_visible,
                scrubber(&mut self.start_trim, &mut self.end_trim, self.video_length, self.trim_to_end),
            );

            let mut args;
            if ui.button("Trim").clicked() {
                ctx.set_cursor_icon(egui::CursorIcon::Progress);
                if self.picked_path.is_none() {
                    self.show_no_file_error = true;
                } else if self.output_location.is_none() {
                    self.show_no_name_error = true;
                } else {
                    self.trim_can_continue = true;
                }

                if self.trim_can_continue {
                    let path = self.picked_path.as_ref().unwrap();
                    let time_start = &num_to_time(self.start_trim as f32);
                    let time_end = &num_to_time(self.end_trim as f32);
                    let output = self.output_location.as_ref().unwrap();
                    if !self.slow_trim {
                        args = vec!["-ss", time_start, "-to", time_end, "-i", path, "-c", "copy", output];
                    } else {
                        // TODO: make async
                        args = vec!["-i", path, "-ss", time_start, "-t", time_end, "-async", "1", output];
                    }
                    if self.overwrite {
                        args.push("-y");
                    }
                    if self.trim_to_end {
                        if !self.slow_trim {
                            args.remove(2);
                            args.remove(2);
                        } else {
                            args.remove(4);
                            args.remove(4);
                        }
                    }
                    let cmd = Command::new("ffmpeg").args(args).output().expect("Error when trimming video!");
                    if !self.ffmpeg_gen_output_made {
                        self.ffmpeg_gen_output_made = true;
                        self.ffmpeg_gen_output = Some(String::from_utf8_lossy(&cmd.stderr).into_owned());
                    }

                    if cmd.status.success() {
                        self.trim_finished = true;
                    }
                }
            }

            if self.show_no_file_error {
                egui::Window::new("Error!").collapsible(false).resizable(false).show(ctx, |ui| {
                    ui.colored_label(Color32::LIGHT_RED, "You need to provide a path to the video you want to trim!");
                    if ui.button("Ok").clicked() {
                        self.show_no_file_error = false;
                    }
                });
            }

            if self.show_no_name_error {
                egui::Window::new("Error!").collapsible(false).resizable(false).show(ctx, |ui| {
                    ui.colored_label(Color32::LIGHT_RED, "You need to provide an output path for the trimmed video!");
                    if ui.button("Ok").clicked() {
                        self.show_no_name_error = false;
                    }
                });
            }

            if self.trim_finished {
                egui::Window::new("Output")
                    .default_height(300.0)
                    .collapsible(false)
                    .resizable(true)
                    .constrain(false)
                    .show(ctx, |ui| {
                        ui.heading("Trim Complete!");
                        egui::ScrollArea::vertical().max_height(200.0).stick_to_bottom(true).show(ui, |ui| {
                            if let Some(text) = &self.ffmpeg_gen_output {
                                ui.label(text);
                            }
                        });
                        ui.separator();
                        if ui.button("Close").clicked() {
                            *self = Self::default();
                        }
                    });
            }
        });
    }
}

fn num_to_time(n: f32) -> String {
    let hours = n as i32 / (60 * 60);
    let mins = (n as i32 / 60) % 60;
    let secs = n % 60.0;
    // add setting for millisecond precision?
    format!("{hours:02}:{mins:02}:{secs:02.2}")
}

// custom scrubber widget

pub fn scroll_scrubber(ui: &mut egui::Ui, start: &mut f32, end: &mut f32, video_length: u32, to_end: bool) -> egui::Response {
    let scrub_size = egui::vec2(360.0, 36.0);
    let drag_size = egui::vec2(360.0, 20.0);

    let trim_step = video_length as f32 / 380.0;

    let (rect, response) = ui.allocate_exact_size(scrub_size, egui::Sense::focusable_noninteractive());
    let (left_drag_rect, mut left_response) = ui.allocate_exact_size(drag_size, egui::Sense::drag());
    let (right_drag_rect, mut right_response) = ui.allocate_exact_size(drag_size, egui::Sense::drag());

    let size = egui::vec2(10.0, 20.0);
    let half_width = size.x / 2.0;
    let mut left_drag_scrub_rect = egui::Rect::from_center_size(egui::pos2(rect.left() + half_width, left_drag_rect.center().y), size);
    let mut right_drag_scrub_rect = egui::Rect::from_center_size(egui::pos2(rect.right() - half_width, right_drag_rect.center().y), size);

    left_response = left_response.on_hover_and_drag_cursor(egui::CursorIcon::ResizeHorizontal);
    if left_response.dragged() {
        if left_response.drag_delta().x > 0.0 {
            *start += trim_step * left_response.drag_delta().x;
        }
        if left_response.drag_delta().x < 0.0 {
            *start -= f32::abs(trim_step * left_response.drag_delta().x);
        }
        left_response.mark_changed();
    }

    right_response = right_response.on_hover_and_drag_cursor(egui::CursorIcon::ResizeHorizontal);
    if right_response.dragged() && !to_end {
        if right_response.drag_delta().x > 0.0 {
            *end += trim_step * right_response.drag_delta().x;
        }
        if right_response.drag_delta().x < 0.0 {
            *end -= f32::abs(trim_step * right_response.drag_delta().x);
        }
        right_response.mark_changed();
    }

    if *start < 0.0 {
        *start = 0.0;
    }
    if *end < 0.0 {
        *end = 0.0;
    }
    if *end > video_length as f32 || (*end != video_length as f32 && to_end) {
        *end = video_length as f32;
    }

    let mut scrub_rect = rect;

    let move_start = *start as f32 / trim_step as f32;
    let mut move_end = *end as f32 / trim_step as f32;
    scrub_rect.set_left(move_start);
    scrub_rect.set_right(move_end);

    if scrub_rect.left() < rect.left() {
        scrub_rect.set_left(rect.left());
    }
    if scrub_rect.right() > rect.right() {
        scrub_rect.set_right(rect.right());
    }

    if right_drag_scrub_rect.right() > rect.right() {
        move_end -= rect.right() - right_drag_scrub_rect.right();
    }

    left_drag_scrub_rect.set_center(egui::pos2(move_start + half_width, left_drag_scrub_rect.center().y));
    right_drag_scrub_rect.set_center(egui::pos2(move_end - half_width, right_drag_scrub_rect.center().y));

    if left_drag_scrub_rect.left() < left_drag_rect.left() {
        left_drag_scrub_rect.set_center(egui::pos2(
            left_drag_scrub_rect.center().x + left_drag_rect.left(),
            left_drag_scrub_rect.center().y,
        ));
    }
    if left_drag_scrub_rect.right() > left_drag_rect.right() {
        left_drag_scrub_rect.set_center(egui::pos2(left_drag_rect.right() - half_width, left_drag_scrub_rect.center().y));
    }
    if right_drag_scrub_rect.right() > right_drag_rect.right() {
        right_drag_scrub_rect.set_center(egui::pos2(
            right_drag_rect.right() - (right_drag_scrub_rect.width() / 2.0),
            right_drag_scrub_rect.center().y,
        ));
    }
    if right_drag_scrub_rect.left() < right_drag_rect.left() {
        right_drag_scrub_rect.set_center(egui::pos2(left_drag_rect.left() + half_width, right_drag_scrub_rect.center().y));
    }

    if ui.is_rect_visible(rect) {
        ui.painter()
            .rect(rect, 0.0, Color32::DARK_GRAY, egui::Stroke::new(1.0, Color32::DARK_GRAY));
        ui.painter().rect_filled(scrub_rect, 0.0, Color32::LIGHT_YELLOW);
        ui.painter().rect_stroke(left_drag_rect, 0.0, egui::Stroke::new(1.0, Color32::LIGHT_GRAY));
        ui.painter()
            .rect_stroke(right_drag_rect, 0.0, egui::Stroke::new(1.0, Color32::LIGHT_GRAY));
        ui.painter().rect_filled(left_drag_scrub_rect, 0.0, Color32::WHITE);
        ui.painter().rect_filled(right_drag_scrub_rect, 0.0, Color32::WHITE);
    }

    response
}

pub fn scrubber<'a>(start: &'a mut f32, end: &'a mut f32, video_length: u32, to_end: bool) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| scroll_scrubber(ui, start, end, video_length, to_end)
}
