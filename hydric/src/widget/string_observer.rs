use egui::TextBuffer;
use std::ops::Range;

pub fn string_observer<F: Fn(Option<String>) -> String>(
    get_set: F,
    initial: String,
) -> StringObserver<F> {
    StringObserver {
        get_set,
        owned: initial,
    }
}

pub struct StringObserver<F: Fn(Option<String>) -> String> {
    get_set: F,
    owned: String, // only used for as_str.
}

impl<F: Fn(Option<String>) -> String> TextBuffer for StringObserver<F> {
    fn is_mutable(&self) -> bool {
        true
    }

    fn as_str(&self) -> &str {
        self.owned.as_str()
    }

    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        let mut buffer = (self.get_set)(None).to_owned();
        let result = buffer.insert_text(text, char_index);
        (self.get_set)(Some(buffer.clone()));
        self.owned = buffer.clone();
        result
    }

    fn delete_char_range(&mut self, char_range: Range<usize>) {
        let mut buffer = (self.get_set)(None).to_owned();
        buffer.delete_char_range(char_range);
        (self.get_set)(Some(buffer.clone()));
        self.owned = buffer.clone();
    }

    fn clear(&mut self) {
        (self.get_set)(Some(String::new()));
        self.owned = String::new();
    }

    fn replace_with(&mut self, text: &str) {
        (self.get_set)(Some(text.to_string()));
        self.owned = text.to_string();
    }

    fn take(&mut self) -> String {
        let buffer = (self.get_set)(None).to_owned();
        self.clear();
        buffer
    }
}
