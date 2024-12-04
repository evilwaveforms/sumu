use egui::*;
use std::cell::RefCell;

pub struct Sumu {
    redo_history: RefCell<Vec<Action>>,
    actions: RefCell<Vec<Action>>,
    latest: bool,
    curr_action_type: ActionType,
    curr_stroke: Stroke,
    img_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Action {
    lines: RefCell<Vec<Pos2>>,
    stroke: Stroke,
    action_type: ActionType,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActionType {
    Paint,
    Erase,
}

impl Default for Sumu {
    fn default() -> Self {
        Self {
            redo_history: Default::default(),
            actions: Default::default(),
            latest: true,
            curr_action_type: ActionType::Paint,
            curr_stroke: Stroke::new(1.0, Color32::from_rgb(136, 8, 8)),
            img_path: None,
        }
    }
}

impl Default for Action {
    fn default() -> Self {
        Self {
            lines: Default::default(),
            // stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            stroke: Stroke::new(1.0, Color32::from_rgb(136, 8, 8)),
            action_type: ActionType::Paint,
        }
    }
}

impl Sumu {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    pub fn paint(&mut self, canvas: &mut Response, from_screen: emath::RectTransform) {
        let mut current_action = self.actions.borrow_mut();
        if canvas.clicked() || canvas.dragged() {
            if current_action.is_empty() {
                let new_action = Action {
                    action_type: self.curr_action_type,
                    stroke: self.curr_stroke,
                    ..Default::default()
                };
                current_action.push(new_action);
            }
            self.redo_history.get_mut().clear();
            self.latest = true;
        }
        if !current_action.is_empty() {
            let curr_action = current_action.last_mut().unwrap();
            curr_action.action_type = self.curr_action_type.clone();
            curr_action.stroke = self.curr_stroke.clone();

            if let Some(pointer_pos) = canvas.interact_pointer_pos() {
                let canvas_pos = from_screen * pointer_pos;
                if curr_action.lines.borrow().last() != Some(&canvas_pos) {
                    curr_action.lines.borrow_mut().push(canvas_pos);
                    canvas.mark_changed();
                }
            } else if !curr_action.lines.borrow().is_empty() {
                let new_action = Action {
                    action_type: self.curr_action_type,
                    stroke: self.curr_stroke,
                    ..Default::default()
                };
                current_action.push(new_action);
                canvas.mark_changed();
            }
        }
    }

    pub fn undo(&mut self) {
        if self.actions.borrow().len() > 0 {
            if self.redo_history.get_mut().len() == 0 && self.latest == true {
                self.actions.get_mut().pop();
                self.latest = false;
            }
            let last_action = self.actions.get_mut().pop();
            self.redo_history.get_mut().push(last_action.unwrap());
        }
    }

    pub fn redo(&mut self) {
        if self.redo_history.borrow().len() > 0 {
            let redo = self.redo_history.get_mut().pop();
            self.actions.get_mut().push(redo.unwrap());
        }
    }

    pub fn handle_save(&self, ctx: &Context, canvas: Response) {
        let screenshot = ctx.input(|i| {
            for event in &i.raw.events {
                if let egui::Event::Screenshot { image, .. } = event {
                    return Some(image.clone());
                }
            }
            None
        });

        if let (Some(screenshot), area) = (screenshot, canvas.rect) {
            if let Some(mut path) = rfd::FileDialog::new().save_file() {
                path.set_extension("png");
                let ppp = ctx.pixels_per_point();
                let img = screenshot.region(&area, Some(ppp));
                image::save_buffer(
                    &path,
                    img.as_raw(),
                    img.width() as u32,
                    img.height() as u32,
                    image::ColorType::Rgba8,
                )
                .unwrap();
                eprintln!("Image saved to {path:?}.");
            }
        }
    }
}

impl eframe::App for Sumu {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.visuals_mut().selection.bg_fill = Color32::from_rgb(136, 8, 8);
            ui.visuals_mut().selection.stroke = Stroke::new(1.0, Color32::BLACK);

            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.img_path = Some(path.display().to_string());
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
                        ui.close_menu();
                    }
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Clear").clicked() {
                        self.actions.get_mut().clear();
                        self.redo_history.get_mut().clear();
                        self.img_path = None;
                        ui.close_menu();
                    }
                });

                ui.add_space(8.0);
                if ui
                    .add_enabled(self.actions.borrow().len() > 0, egui::Button::new("⮪"))
                    .clicked()
                {
                    self.undo();
                }
                if ui
                    .add_enabled(self.redo_history.borrow().len() > 0, egui::Button::new("⮫"))
                    .clicked()
                {
                    self.redo();
                }

                ui.add_space(8.0);
                ui.selectable_value(&mut self.curr_action_type, ActionType::Erase, "⬜");
                ui.selectable_value(&mut self.curr_action_type, ActionType::Paint, "✏");
                ui.add(&mut self.curr_stroke);

                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                    egui::widgets::global_dark_light_mode_buttons(ui)
                });
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.heading("𝔰𝔲𝔪𝔲");
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (mut canvas, painter) =
                    ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());
                let to_screen = emath::RectTransform::from_to(
                    Rect::from_min_size(Pos2::ZERO, canvas.rect.square_proportions()),
                    canvas.rect,
                );
                let from_screen = to_screen.inverse();

                if let Some(img_path) = &self.img_path {
                    egui::Image::new("file://".to_owned() + img_path).paint_at(ui, canvas.rect);
                }

                if ctx.input(|input| input.modifiers.matches_logically(Modifiers::CTRL)) {
                    if ctx.input(|input| input.key_pressed(Key::Z)) {
                        self.undo();
                    }
                    if ctx.input(|input| input.key_pressed(Key::R)) {
                        self.redo();
                    }
                } else if canvas.hovered() {
                    self.paint(&mut canvas, from_screen);
                }

                let current_action = self.actions.borrow_mut();
                if !current_action.is_empty() {
                    let shapes = current_action
                        .iter()
                        .filter(|action| action.lines.borrow().len() >= 2)
                        .map(|action| {
                            let points: Vec<Pos2> = action
                                .lines
                                .borrow()
                                .iter()
                                .map(|p| to_screen * *p)
                                .collect();

                            if action.action_type == ActionType::Erase {
                                let bg_color = ui.visuals().extreme_bg_color;
                                let stroke = Stroke::new(10.0, bg_color);
                                egui::Shape::line(points, stroke)
                            } else {
                                egui::Shape::line(points, action.stroke)
                            }
                        });
                    painter.extend(shapes);
                }

                self.handle_save(&ctx, canvas);
            });
        });
    }
}
