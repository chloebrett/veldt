pub fn for_each_with_separator<T, F, S>(
    ui: &mut egui::Ui,
    iter: impl IntoIterator<Item = T>,
    mut for_each: F,
    mut separator: S,
) where
    F: FnMut(&mut egui::Ui, T),
    S: FnMut(&mut egui::Ui),
{
    let mut iter = iter.into_iter().peekable();

    while let Some(item) = iter.next() {
        for_each(ui, item);

        // Only add separator if there's another item
        if iter.peek().is_some() {
            separator(ui);
        };
    }
}
