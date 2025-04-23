use egui::Ui;

pub trait View {
    // TODO consider returning a Response to handle other interactions.
    fn ui(&mut self, ui: &mut Ui);
}
