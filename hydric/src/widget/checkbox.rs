use egui::Ui;

pub fn checkbox(ui: &mut Ui, value: bool, mut setter: impl FnMut(bool), text: &str) {
    let mut temp = value;
    ui.checkbox(&mut temp, text);
    if value != temp {
        setter(temp);
    }
}
