pub const fn opacity_percentage_to_alpha(opacity_percentage: f32) -> u8 {
    ((opacity_percentage / 100.0) * 255.0) as u8
}

pub const fn choose_black_white_based_on_contrast(colour: [u8; 3]) -> [u8; 3] {
    let (r, g, b) = (colour[0] as f32, colour[1] as f32, colour[2] as f32);
    let perceived_luminance = 0.299 * r + 0.587 * g + 0.114 * b; // formula here: https://www.w3.org/TR/AERT/#color-contrast
    if perceived_luminance > 186.0 {
        return [0, 0, 0]
    } else {
        return [255, 255, 255]
    }
}