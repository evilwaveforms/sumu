use egui::*;
use std::cell::RefCell;

pub struct Sumu {
    redo_history: RefCell<Vec<Action>>,
    actions: RefCell<Vec<Action>>,
}

pub struct Action {
    lines: RefCell<Vec<Pos2>>,
    stroke: Stroke,
}

impl Default for Sumu {
    fn default() -> Self {
        Self {
            redo_history: Default::default(),
            actions: Default::default(),
        }
    }
}

impl Default for Action {
    fn default() -> Self {
        Self {
            lines: Default::default(),
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
        }
    }
}

impl Sumu {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );
        let from_screen = to_screen.inverse();

        if self.actions.borrow().is_empty() {
            let action = Action::default();
            self.actions.borrow_mut().push(action);
        }

        let mut current_action = self.actions.borrow_mut();

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let canvas_pos = from_screen * pointer_pos;
            if current_action.last().unwrap().lines.borrow().last() != Some(&canvas_pos) {
                current_action
                    .last()
                    .unwrap()
                    .lines
                    .borrow_mut()
                    .push(canvas_pos);
                response.mark_changed();
            }
        } else if !current_action.last().unwrap().lines.borrow().is_empty() {
            current_action.push(Default::default());
            response.mark_changed();
        }

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
                let shape = egui::Shape::line(points, action.stroke);
                shape
            });
        painter.extend(shapes);
        response
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
                    .add_enabled(last_line > 1, egui::Button::new("⮪"))
                    .clicked()
                {
                    let mut last_action = self.actions.borrow_mut().pop();
                    let lines = &self.actions.get_mut().get_mut(last_line - 2).unwrap().lines;
                    last_action.as_mut().unwrap().lines = lines.clone();
                    self.redo_history.get_mut().push(last_action.unwrap());
                    lines.borrow_mut().clear();
                }
                if ui
                    .add_enabled(self.redo_history.borrow().len() > 0, egui::Button::new("⮫"))
                    .clicked()
                {
                    let redo = self.redo_history.get_mut().pop();
                    self.actions.get_mut().push(redo.unwrap());
                }
            });
        });
        // println!("{:?}", ctx.input(|i| i.pointer.to_owned()));
        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            // ui.heading("𝔰𝔲𝔪𝔲");
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let canvas = self.ui_content(ui);
                if canvas.dragged() || canvas.clicked() {
                    self.redo_history.get_mut().clear();
                }
            });
        });
    }
}
