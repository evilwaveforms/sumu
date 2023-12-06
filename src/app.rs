use egui::*;
use std::cell::RefCell;

pub struct Sumu {
    lines: RefCell<Vec<Vec<Pos2>>>,
    stroke: Stroke,
}

impl Default for Sumu {
    fn default() -> Self {
        Self {
            lines: Default::default(),
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
        }
    }
}

impl Sumu {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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

        if self.lines.borrow().is_empty() {
            self.lines.borrow_mut().push(vec![]);
        }

        let current_line = self.lines.get_mut().last_mut().unwrap();

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let canvas_pos = from_screen * pointer_pos;
            if current_line.last() != Some(&canvas_pos) {
                current_line.push(canvas_pos);
                response.mark_changed();
            }
        } else if !current_line.is_empty() {
            self.lines.borrow_mut().push(vec![]);
            response.mark_changed();
        }

        // let mut shapes = self
        //     .lines
        //     .iter()
        //     .filter(|line| line.len() >= 2)
        //     .map(|line| {
        //         let points: Vec<Pos2> = line.iter().map(|p| to_screen * *p).collect();
        //         let shape = egui::Shape::line(points, self.stroke);
        //         shape
        //     });

        for (idx, line) in self.lines.borrow().iter().enumerate() {
            if line.len() >= 2 {
                let points: Vec<egui::Pos2> = line.iter().map(|p| to_screen * *p).collect();
                let shape = egui::Shape::line(points, self.stroke);
                painter.add(shape);
            }
        }
        // painter.extend(shapes);
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
                ui.add_space(16.0);
                egui::widgets::global_dark_light_mode_buttons(ui);

                if ui
                    .add_enabled(self.lines.get_mut().len() > 1, egui::Button::new("⮪"))
                    .clicked()
                {
                    let last_line = self.lines.get_mut().len() - 2;
                    self.lines.get_mut().get_mut(last_line).unwrap().clear();
                    self.lines.get_mut().pop();
                }
                if ui.add_enabled(false, egui::Button::new("⮫")).clicked() {
                    unreachable!();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            // ui.heading("𝔰𝔲𝔪𝔲");
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.ui_content(ui);
            });
        });
    }
}
