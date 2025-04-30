use super::TextRotation;
use crate::view::View;
use crate::widget::text_rotator;
use egui::{Color32, FontId, Frame, Stroke, Ui, Vec2};

pub enum TabOrientation {
    Left,
    Right,
    Top,
}

type OnTabClick = Box<dyn Fn(&mut egui::Ui, usize) + Send + Sync + 'static>;

pub struct TabDisplay<'a> {
    active_tab: usize,
    tab_headings: Vec<&'a str>,
    orientation: TabOrientation,
    handle_click: OnTabClick,
}

impl<'a> TabDisplay<'a> {
    pub fn new(
        active_tab: usize,
        tab_headings: Vec<&'a str>,
        orientation: TabOrientation,
        handle_click: OnTabClick,
    ) -> Self {
        Self {
            active_tab,
            tab_headings,
            orientation,
            handle_click,
        }
    }
}

impl View for TabDisplay<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let tab_headings = &self.tab_headings;
        let orientation = &self.orientation;
        let num_tabs = tab_headings.len();

        const TAB_FONT_SIZE: f32 = 14.0;
        const ACTIVE_TAB_COLOUR: Color32 = Color32::from_rgb(30, 30, 30);
        const INACTIVE_TAB_COLOUR: Color32 = Color32::from_rgb(50, 50, 50);

        // Calculate the tab sizes.
        let mut tab_sizes: Vec<Vec2> = Vec::with_capacity(num_tabs);
        let font_id = FontId::proportional(TAB_FONT_SIZE);
        for heading in tab_headings {
            let galley = ui.fonts(|fonts| {
                fonts.layout_no_wrap(heading.to_string(), font_id.clone(), Color32::WHITE)
            }); // assign arbitrary font colour we're only concerned about the galley size
            let padded_size = galley.size() + egui::Vec2::splat(4.0);
            tab_sizes.push(padded_size);
        }

        let tab_rounding = match orientation {
            TabOrientation::Left => egui::CornerRadius {
                nw: 5,
                ne: 0,
                sw: 5,
                se: 0,
            },
            TabOrientation::Right => egui::CornerRadius {
                nw: 0,
                ne: 5,
                sw: 0,
                se: 5,
            },
            TabOrientation::Top => egui::CornerRadius {
                nw: 5,
                ne: 5,
                sw: 0,
                se: 0,
            },
        };
        ui.vertical(|ui| {
            ui.add_space(8.0);
            for i in 0..tab_headings.len() {
                let frame = if i == self.active_tab {
                    Frame::new()
                        .fill(ACTIVE_TAB_COLOUR)
                        .stroke(Stroke::new(1.0, ACTIVE_TAB_COLOUR))
                        .corner_radius(tab_rounding)
                        .inner_margin(4.0)
                } else {
                    Frame::new()
                        .fill(INACTIVE_TAB_COLOUR)
                        .stroke(Stroke::new(1.0, INACTIVE_TAB_COLOUR))
                        .corner_radius(tab_rounding)
                        .inner_margin(4.0)
                };

                let response = frame
                    .show(ui, |ui| {
                        let text_rotation = match orientation {
                            TabOrientation::Left => TextRotation::Anticlockwise90,
                            TabOrientation::Right => TextRotation::Clockwise90,
                            TabOrientation::Top => TextRotation::Neutral,
                        };
                        text_rotator(
                            ui,
                            tab_headings[i],
                            TAB_FONT_SIZE,
                            text_rotation,
                            Color32::WHITE,
                        )
                    })
                    .inner;
                ui.add_space(8.0);
                if response.clicked() {
                    (self.handle_click)(ui, i)
                }
            }
        });
    }
}
