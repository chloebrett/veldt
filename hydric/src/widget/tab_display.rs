use egui::{Color32, FontId, Pos2, Shape, Stroke, Ui, epaint, Frame, Vec2};
use crate::widget::text_rotator;
use crate::view::View;
use super::TextRotation;
use std::fmt;




pub enum TabOrientation {
    Left, // tabs appear on the left
    Right, // tabs appear on the right
    Top, // tabs appear on top
}

// The active tab is stored in mut memory and since mut memory needs a unique id for each item that it stores we define an enum to prevent any double ups from happening
pub enum TabIdentifier {
    SubsynthLfo,
    SubsynthEnv,
}

impl fmt::Display for TabIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TabIdentifier::SubsynthLfo => write!(f, "subsynth lfo tabs"),
            TabIdentifier::SubsynthEnv => write!(f, "subsynth env tabs"),
        }
    }
}

pub struct TabDisplay<'a>{
    id: TabIdentifier,
    tab_headings: Vec<&'a str>,
    tab_contents: Vec<Box<dyn View>>, // Vector of content rendering functions
    orientation: TabOrientation,
}

impl<'a> TabDisplay<'a> {
    pub fn new(
        id: TabIdentifier,
        tab_headings: Vec<&'a str>,
        tab_contents: Vec<Box<dyn View>>, 
        orientation: TabOrientation
    ) -> Self {
        Self {
            id,
            tab_headings,
            tab_contents,
            orientation,
        }
    }
}

impl View for TabDisplay<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let id = &self.id;
        let tab_headings = &self.tab_headings;
        let tab_contents = &self.tab_contents;
        let orientation = &self.orientation;
        let num_tabs = tab_headings.len();
        assert_eq!(num_tabs, tab_contents.len(), "Number of tab headings must match number of tab contents");

        let memory_id = ui.make_persistent_id(id.to_string());
        
        // Retrieve the active tab from memory, or default to 0
        let mut active_tab = ui.memory_mut(|mem: &mut egui::Memory| *mem.data.get_temp_mut_or_default::<usize>(memory_id));
        
        // Ensure the active tab is valid
        if active_tab >= num_tabs && num_tabs > 0 {
            let new_active_tab = 0 as usize;
            ui.memory_mut(|mem| mem.data.insert_temp(memory_id, new_active_tab));
            active_tab = new_active_tab;
        }


        const TAB_FONT_SIZE: f32 = 14.0;

        // Calculate the tab sizes.
        let mut tab_sizes: Vec<Vec2> = Vec::with_capacity(num_tabs);
        let font_id = FontId::proportional(TAB_FONT_SIZE);
        for heading in tab_headings {
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
                for heading in tab_headings {
                    let frame = egui::Frame::new()
                        .fill(egui::Color32::from_rgb(240, 240, 240))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK))
                        .corner_radius(tab_rounding);
                    frame.show(ui, |ui| {
                        text_rotator(
                            ui,
                            heading,
                            TAB_FONT_SIZE,
                            TextRotation::Anticlockwise90,
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
}