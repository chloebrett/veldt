use egui::Ui;
use state::Store;

pub trait View {
    // TODO consider returning a Reponse to handle other interactions.
    fn ui(&self, store: &Store, ui: &mut Ui);
}
