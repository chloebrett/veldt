use egui::Ui;

// Note: use an egui::widgets::Widget instead if you want to return a Response.
pub trait View {
    fn ui(&mut self, ui: &mut Ui);
}
