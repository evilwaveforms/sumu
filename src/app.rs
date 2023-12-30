use egui::*;
use std::cell::RefCell;

pub struct Sumu {
    redo_history: RefCell<Vec<Action>>,
    actions: RefCell<Vec<Action>>,
    latest: bool,
    curr_action_type: ActionType,
    curr_stroke: Stroke,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Action {
    lines: RefCell<Vec<Pos2>>,
    stroke: Stroke,
    action_type: ActionType,
}

#[derive(Clone, Debug, PartialEq)]
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

    pub fn ui_content(&mut self, ui: &mut Ui) -> (egui::Response, egui::Painter) {
        ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag())
    }

    pub fn paint(&mut self, canvas: &mut Response, from_screen: emath::RectTransform) {
        let mut current_action = self.actions.borrow_mut();
        if canvas.clicked() || canvas.dragged() {
            if current_action.is_empty() {
                let mut new_action = Action::default();
                new_action.action_type = self.curr_action_type.clone();
                new_action.stroke = self.curr_stroke.clone();
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
                let mut new_ac = Action::default();
                new_ac.action_type = self.curr_action_type.clone();
                new_ac.stroke = self.curr_stroke.clone();
                current_action.push(new_ac);
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
}

impl eframe::App for Sumu {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.visuals_mut().selection.bg_fill = Color32::from_rgb(136, 8, 8);
            ui.visuals_mut().selection.stroke = Stroke::new(1.0, Color32::BLACK);

            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Clear").clicked() {
                        self.actions.get_mut().clear();
                        self.redo_history.get_mut().clear();
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
                egui::stroke_ui(ui, &mut self.curr_stroke, "Stroke");

                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                    egui::widgets::global_dark_light_mode_buttons(ui)
                });
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            // ui.heading("𝔰𝔲𝔪𝔲");
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (mut canvas, painter) = self.ui_content(ui);
                let to_screen = emath::RectTransform::from_to(
                    Rect::from_min_size(Pos2::ZERO, canvas.rect.square_proportions()),
                    canvas.rect,
                );
                let from_screen = to_screen.inverse();

                if ctx.input(|input| input.modifiers.matches(Modifiers::CTRL)) {
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
            });
        });
    }
}
