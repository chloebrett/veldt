use dasp_ring_buffer::Fixed;
use dasp_rms::Rms;

/// Find the log10 of the Root Mean Squared of a window of audio.
/// Return units are dB.
fn detect_log_rms(window: Vec<f32>) -> f32 {
    let rms: Rms<f32, Vec<f32>> = Rms::new(Fixed::from(window));
    rms.current().log10()
}

/// Convert a stereo signal into two vecs for left and right.
fn split_stereo_audio(stereo_audio: Vec<[f32; 2]>) -> (Vec<f32>, Vec<f32>) {
    let mut left_audio = vec![];
    let mut right_audio = vec![];
    for [left, right] in stereo_audio {
        left_audio.push(left);
        right_audio.push(right);
    }
    (left_audio, right_audio)
}

/// Calculate the audio level of stereo audio in dB.
pub fn find_audio_level(stereo_audio: Vec<[f32; 2]>) -> (f32, f32) {
    let (left_audio, right_audio) = split_stereo_audio(stereo_audio);
    let left_rms = detect_log_rms(left_audio);
    let right_rms = detect_log_rms(right_audio);
    (left_rms, right_rms)
}
