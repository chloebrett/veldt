use egui::{Response, Ui, WidgetText};

/// Variant of egui::Ui::selectable_value which takes a get/set function instead of a mutable
/// variable. https://docs.rs/egui/latest/src/egui/ui.rs.html#2047
pub fn selectable_value<Value: PartialEq>(
    ui: &mut Ui,
    mut get_set_value: impl FnMut(Option<Value>) -> Value,
    selected_value: Value,
    text: impl Into<WidgetText>,
) -> Response {
    let current_value = get_set_value(None);
    let mut response = ui.selectable_label(current_value == selected_value, text);

    if response.clicked() && current_value != selected_value {
        get_set_value(Some(selected_value));

        response.mark_changed();
    }

    response
}
