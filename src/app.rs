use egui::*;
use std::cell::RefCell;

pub struct Sumu {
    redo_history: RefCell<Vec<Action>>,
    actions: RefCell<Vec<Action>>,
    latest: bool,
    curr_action: Action,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Paint {
    lines: RefCell<Vec<Pos2>>,
    stroke: Stroke,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Paint(Paint),
    Erase(Paint),
}

impl Default for Sumu {
    fn default() -> Self {
        Self {
            redo_history: Default::default(),
            actions: Default::default(),
            latest: true,
            curr_action: Action::default(),
        }
    }
}

impl Default for Action {
    fn default() -> Self {
        Action::Paint(Paint::default())
    }
}

impl Default for Paint {
    fn default() -> Self {
        Self {
            lines: Default::default(),
            // stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            stroke: Stroke::new(1.0, Color32::RED),
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

    pub fn paint(
        &mut self,
        canvas: &mut Response,
        from_screen: emath::RectTransform,
        action: Action,
    ) {
        let mut current_action = self.actions.borrow_mut();
        if canvas.clicked() || canvas.dragged() {
            if current_action.is_empty() {
                current_action.push(action.clone());
            }
            self.redo_history.get_mut().clear();
            self.latest = true;
        }
        if !current_action.is_empty() {
            let curr_action = match current_action.last().unwrap() {
                Action::Paint(paint) => paint,
                Action::Erase(erase) => erase,
            };
            if let Some(pointer_pos) = canvas.interact_pointer_pos() {
                let canvas_pos = from_screen * pointer_pos;
                if curr_action.lines.borrow().last() != Some(&canvas_pos) {
                    curr_action.lines.borrow_mut().push(canvas_pos);
                    canvas.mark_changed();
                }
            } else if !curr_action.lines.borrow().is_empty() {
                current_action.push(action.clone());
                canvas.mark_changed();
            }
        }
    }
}

impl eframe::App for Sumu {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
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
                    }
                });

                ui.add_space(16.0);
                egui::widgets::global_dark_light_mode_buttons(ui);

                let last_line = self.actions.borrow().len();

                if ui
                    .add_enabled(last_line > 0, egui::Button::new("⮪"))
                    .clicked()
                {
                    if self.redo_history.get_mut().len() == 0 && self.latest == true {
                        self.actions.get_mut().pop();
                        self.latest = false;
                    }
                    let last_action = self.actions.get_mut().pop();
                    self.redo_history.get_mut().push(last_action.unwrap());
                }
                if ui
                    .add_enabled(self.redo_history.borrow().len() > 0, egui::Button::new("⮫"))
                    .clicked()
                {
                    let redo = self.redo_history.get_mut().pop();
                    self.actions.get_mut().push(redo.unwrap());
                }

                let brush = match self.curr_action.clone() {
                    Action::Paint(paint) => paint,
                    Action::Erase(erase) => erase,
                };
                ui.selectable_value(&mut self.curr_action, Action::Erase(brush.to_owned()), "⬜");
                ui.selectable_value(&mut self.curr_action, Action::Paint(brush.to_owned()), "✏");

                match &mut self.curr_action {
                    Action::Paint(paint) => egui::stroke_ui(ui, &mut paint.stroke, "Stroke"),
                    _ => (),
                };
            });
        });
        // println!("{:?}", ctx.input(|i| i.pointer.to_owned()));
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

                // TODO: Change this for shortcut usage?
                if canvas.hovered() {
                    self.paint(&mut canvas, from_screen, self.curr_action.clone());
                }
                let current_action = self.actions.borrow_mut();
                if !current_action.is_empty() {
                    let shapes = current_action
                        .iter()
                        .filter(|action| match action {
                            Action::Paint(paint) | Action::Erase(paint) => {
                                paint.lines.borrow().len() >= 2
                            }
                        })
                        .map(|action| match action {
                            Action::Paint(paint) | Action::Erase(paint) => {
                                let points: Vec<Pos2> = paint
                                    .lines
                                    .borrow()
                                    .iter()
                                    .map(|p| to_screen * *p)
                                    .collect();

                                match action {
                                    Action::Erase(_) => {
                                        let bg_color = ui.visuals().extreme_bg_color;
                                        let stroke = Stroke::new(10.0, bg_color);
                                        egui::Shape::line(points, stroke)
                                    }
                                    _ => egui::Shape::line(points, paint.stroke),
                                }
                            }
                        });
                    painter.extend(shapes);
                }
            });
        });
    }
}
