use ecolor::Color32;
use shared::model::Colour;

pub const fn opacity_percentage_to_alpha(opacity_percentage: f32) -> u8 {
    ((opacity_percentage / 100.0) * 255.0) as u8
}

pub const fn choose_black_white_based_on_contrast(colour: Colour) -> Colour {
    // formula here: https://www.w3.org/TR/AERT/#color-contrast
    let perceived_luminance = 0.299 * colour.r + 0.587 * colour.g + 0.114 * colour.b;
    if perceived_luminance > 160.0 {
        Colour::black()
    } else {
        Colour::white()
    }
}

pub trait ToEguiColour {
    fn to_egui(&self) -> Color32;

    fn to_egui_additive(&self) -> Color32;

    fn to_egui_unmultiplied(&self, alpha: u8) -> Color32;

    fn from_egui(egui: Color32) -> Self;
}

impl ToEguiColour for Colour {
    fn to_egui(&self) -> Color32 {
        Color32::from_rgb(self.r as u8, self.g as u8, self.b as u8)
    }

    fn to_egui_additive(&self) -> Color32 {
        Color32::from_rgb_additive(self.r as u8, self.g as u8, self.b as u8)
    }

    fn to_egui_unmultiplied(&self, alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(self.r as u8, self.g as u8, self.b as u8, alpha)
    }

    fn from_egui(egui: Color32) -> Self {
        Colour::from_8bit(egui.r(), egui.g(), egui.b())
    }
}
