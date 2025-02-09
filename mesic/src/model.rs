pub type Volume = f32;
pub type Freq = f32;
pub type Semitones = f32;
pub type Beats = f32;

#[derive(Debug)]
pub struct Track {
    pub bpm: Beats,
    pub synths: Vec<Synth>,
    pub sequences: Vec<Sequence>,
}

#[derive(Debug)]
pub struct Sequence {
    pub offset: Beats,
    pub volume: Volume,
    pub synth_index: usize,
    pub notes: Vec<Note>,
}

#[derive(Debug)]
pub struct Synth {
    pub wave: WaveType,
    pub envelope: AdsrEnvelope,
    pub volume: Volume,
}

#[derive(Debug)]
pub struct Note(pub Semitones, pub Beats);

#[derive(Debug)]
pub struct AdsrEnvelope {
    pub attack: Beats,
    pub decay: Beats,
    pub sustain: Volume,
    pub release: Beats,
}

#[derive(Debug, Clone, Copy)]
pub enum WaveType {
    Sine,
    Square,
    Saw,
    Triangle, // TODO: also add a generator for white noise - but it's not constrained by freq.
}
