use egui::Ui;

pub trait View {
    // TODO consider returning a Response to handle other interactions.
    // TODO: should this be &mut self instead?
    fn ui(&self, ui: &mut Ui);
}
