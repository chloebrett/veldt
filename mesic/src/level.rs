use dasp_ring_buffer::Fixed;
use dasp_rms::Rms;

use crate::split_stereo_audio;

const RMS_WINDOW: usize = 64;

/// Find the log10 of the Root Mean Squared of a window of audio.
/// Return units are dB.
fn detect_log_rms(audio: &[f32]) -> f32 {
    // TODO: Store RMS state to avoid re-creating everying window.
    let window = Fixed::from([0f32; RMS_WINDOW]);
    let mut rms = Rms::new(window);
    for value in audio.iter() {
        rms.next(*value);
    }
    rms.current().log10()
}

/// Calculate the audio level of stereo audio in dB.
pub fn calc_audio_level(stereo_audio: &[[f32; 2]]) -> (f32, f32) {
    let (left_audio, right_audio) = split_stereo_audio(stereo_audio);
    let left_rms = detect_log_rms(left_audio.as_slice());
    let right_rms = detect_log_rms(right_audio.as_slice());
    (left_rms, right_rms)
}
