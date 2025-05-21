use egui::{Checkbox, Ui};

pub fn checkbox<F>(ui: &mut Ui, make_checkbox: F, value: bool, mut setter: impl FnMut(bool))
where
    F: Fn(&mut bool) -> Checkbox,
{
    let mut temp = value;
    ui.add(make_checkbox(&mut temp));
    if value != temp {
        setter(temp);
    }
}
