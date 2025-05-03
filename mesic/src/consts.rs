
pub const SAMPLE_RATE: i32 = 44_100;
pub const SAMPLE_RATE_RECIP: f32 = 1.0 / SAMPLE_RATE as f32;
pub const NYQUIST: i32 = SAMPLE_RATE / 2;
pub const SECONDS_PER_MINUTE: f32 = 60.0;
pub const FFT_SAMPLE_SIZE: usize = 1024;
