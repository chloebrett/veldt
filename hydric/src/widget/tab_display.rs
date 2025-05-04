use super::TextRotation;
use crate::view::View;
use crate::widget::text_rotator;
use egui::{Color32, Frame, Stroke, Ui};

#[derive(Clone)]
pub enum TabOrientation {
    Left,
    _Right,
    _Top,
}

impl From<TabOrientation> for TextRotation {
    fn from(orientation: TabOrientation) -> Self {
        match orientation {
            TabOrientation::Left => TextRotation::Anticlockwise90,
            TabOrientation::_Right => TextRotation::Clockwise90,
            TabOrientation::_Top => TextRotation::Neutral,
        }
    }
}

pub struct TabDisplay<'a, F: Fn(usize)> {
    active_tab: usize,
    tab_headings: Vec<&'a str>,
    orientation: TabOrientation,
    handle_click: F,
}

impl<'a, F: Fn(usize)> TabDisplay<'a, F> {
    pub fn new(
        active_tab: usize,
        tab_headings: Vec<&'a str>,
        orientation: TabOrientation,
        handle_click: F,
    ) -> Self {
        Self {
            active_tab,
            tab_headings,
            orientation,
            handle_click,
        }
    }
}

impl<F: Fn(usize)> View for TabDisplay<'_, F> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            ref tab_headings,
            ref orientation,
            active_tab,
            ref handle_click,
        } = *self;

        const TAB_FONT_SIZE: f32 = 14.0;
        const ACTIVE_TAB_COLOUR: Color32 = Color32::from_gray(30);
        const INACTIVE_TAB_COLOUR: Color32 = Color32::from_gray(50);

        let tab_rounding = match orientation {
            TabOrientation::Left => egui::CornerRadius {
                nw: 5,
                ne: 0,
                sw: 5,
                se: 0,
            },
            TabOrientation::_Right => egui::CornerRadius {
                nw: 0,
                ne: 5,
                sw: 0,
                se: 5,
            },
            TabOrientation::_Top => egui::CornerRadius {
                nw: 5,
                ne: 5,
                sw: 0,
                se: 0,
            },
        };
        ui.vertical(|ui| {
            ui.add_space(8.0);
            for (i, heading) in tab_headings.iter().enumerate() {
                let colour = if i == active_tab {
                    ACTIVE_TAB_COLOUR
                } else {
                    INACTIVE_TAB_COLOUR
                };
                let frame = Frame::new()
                    .fill(colour)
                    .stroke(Stroke::new(1.0, colour))
                    .corner_radius(tab_rounding)
                    .inner_margin(4.0);

                let response = frame
                    .show(ui, |ui| {
                        let text_rotation = orientation.clone().into();
                        text_rotator(ui, heading, TAB_FONT_SIZE, text_rotation, Color32::WHITE)
                    })
                    .inner;
                ui.add_space(8.0);
                if response.clicked() {
                    handle_click(i)
                }
            }
        });
    }
}
