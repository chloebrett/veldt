use egui::{Context, Ui};

pub trait View {
    // TODO consider returning a Response to handle other interactions.
    fn ui(&mut self, ui: &mut Ui);
}

pub trait WindowView {
    fn ui(&mut self, ctx: &Context);
}
