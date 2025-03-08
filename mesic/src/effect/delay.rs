fn apply_delay(dry_signal: Vec<f32>, amplitude: Volume, delay_ms: Milliseconds) -> Vec<f32> {
    // TODO: consider if fractional samples / interpolation make sense here.
    let sample_count = (delay_ms * (SAMPLE_RATE as f32) / 1000.0) as usize;
    let mut wet_signal = vec![0.0; sample_count];

    wet_signal.extend(dry_signal);

    mult(wet_signal, amplitude)
}
