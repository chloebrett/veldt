use std::f32::consts::PI;

pub const SAMPLE_RATE: i32 = 44_100;
pub const RECIP_SAMPLE_RATE: f32 = 1.0 / SAMPLE_RATE as f32;
pub const NYQUIST: i32 = SAMPLE_RATE / 2;
pub const SECONDS_PER_MINUTE: f32 = 60.0;
pub const MS_PER_SECOND: f32 = 1000.0;
pub const FFT_SAMPLE_SIZE: usize = 4096;
pub const CHANNEL_COUNT: usize = 2;

pub const RECIP_PI: f32 = PI.recip();
pub const DOUBLE_RECIP_PI: f32 = 2.0 * RECIP_PI;
