use egui::{Color32, FontId, Pos2, Shape, Stroke, Ui, epaint, Frame, Vec2};
use crate::widget::text_rotator;

use super::TextRotation;

pub enum TabOrientation {
    Left, // tabs appear on the left
    Right, // tabs appear on the right
    Top, // tabs appear on top
}

pub fn tab_display(ui: &mut Ui, num_tabs: usize, tab_headings: Vec<&str>, orientation: TabOrientation) {
    let mut current_tab = 0;
    const TAB_FONT_SIZE: f32 = 14.0;

    // alculate the tab sizes:
    let mut tab_sizes: Vec<Vec2> = Vec::with_capacity(num_tabs);
    let font_id = FontId::proportional(TAB_FONT_SIZE);
    for heading in &tab_headings {
        let galley =
            ui.fonts(|fonts| fonts.layout_no_wrap(heading.to_string(), font_id.clone(), Color32::WHITE)); // assign arbitrary font colour we're only concerned about the galley size
        let padded_size = galley.size() + egui::Vec2::splat(4.0);
        tab_sizes.push(padded_size);
    }

    let tab_rounding = match orientation{
        TabOrientation::Left =>
        egui::CornerRadius {
            nw: 5,
            ne: 0,
            sw: 5,
            se: 0,
        },
        TabOrientation::Right => 
        egui::CornerRadius {
            nw: 0,
            ne: 5,
            sw: 0,
            se: 5,
        },
        TabOrientation::Top =>
        egui::CornerRadius {
            nw: 5,
            ne: 5,
            sw: 0,
            se: 0,
        },
    };
    
    match orientation {
        TabOrientation::Left => {
            for heading in &tab_headings {
                let frame = egui::Frame::new()
                    .fill(egui::Color32::from_rgb(240, 240, 240))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK))
                    .corner_radius(tab_rounding);
                frame.show(ui, |ui| {
                    text_rotator(
                        ui,
                        heading,
                        TAB_FONT_SIZE,
                        TextRotation::Anticlockwise,
                        Color32::BLACK,
                    );
                }
            );
            }
        }
        ,
        TabOrientation::Right => {

        }
        ,
        TabOrientation::Top => {

        },
    }

}