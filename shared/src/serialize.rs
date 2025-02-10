use crate::model::*;
use crate::pmodel::*;

fn map_vec<I, O>(input: Vec<I>) -> Vec<O>
    where I : Into<O> {
    input.into_iter().map(|elem| elem.into()).collect()
}

impl From<TrackProto> for Track {
    fn from(item: TrackProto) -> Self {
        Track {
            bpm: item.bpm,
            synths: map_vec(item.synth),
            sequences: map_vec(item.sequence),
        }
    }
}

impl From<Track> for TrackProto {
    fn from(item: Track) -> Self {
        TrackProto {
            bpm: item.bpm,
            synth: map_vec(item.synths),
            sequence: map_vec(item.sequences),
        }
    }
}

impl From<SequenceProto> for Sequence {
    fn from(item: SequenceProto) -> Self {
        Sequence {
            offset: item.offset,
            volume: item.volume,
            synth_index: item.synth_index as usize,
            notes: map_vec(item.note),
        }
    }
}

impl From<Sequence> for SequenceProto {
    fn from(item: Sequence) -> Self {
        SequenceProto {
            offset: item.offset,
            volume: item.volume,
            synth_index: item.synth_index as u32,
            note: map_vec(item.notes),
        }
    }
}

impl From<SynthProto> for Synth {
    fn from(item: SynthProto) -> Self {
        Synth {
            wave: TryInto::<WaveTypeProto>::try_into(item.wave).unwrap().into(),
            envelope: item.envelope.unwrap().into(),
            volume: item.volume,
        }
    }
}

impl From<Synth> for SynthProto {
    fn from(item: Synth) -> Self {
        SynthProto {
            wave: Into::<WaveTypeProto>::into(item.wave) as i32,
            envelope: Some(item.envelope.into()),
            volume: item.volume,
        }
    }
}

impl From<NoteProto> for Note {
    fn from(item: NoteProto) -> Self {
        Note(item.semitones, item.beats)
    }
}

impl From<Note> for NoteProto {
    fn from(item: Note) -> Self {
        NoteProto {
            semitones: item.0,
            beats: item.1,
        }
    }
}

impl From<AdsrEnvelopeProto> for AdsrEnvelope {
    fn from(item: AdsrEnvelopeProto) -> Self {
        AdsrEnvelope {
            attack: item.attack,
            decay: item.decay,
            sustain: item.sustain,
            release: item.release,
        }
    }
}

impl From<AdsrEnvelope> for AdsrEnvelopeProto {
    fn from(item: AdsrEnvelope) -> Self {
        AdsrEnvelopeProto {
            attack: item.attack,
            decay: item.decay,
            sustain: item.sustain,
            release: item.release,
        }
    }
}

impl From<WaveTypeProto> for WaveType {
    fn from(item: WaveTypeProto) -> Self {
        match item {
            WaveTypeProto::Unknown => panic!(""),
            WaveTypeProto::Sine => WaveType::Sine,
            WaveTypeProto::Square => WaveType::Square,
            WaveTypeProto::Saw => WaveType::Saw,
            WaveTypeProto::Triangle => WaveType::Triangle,
        }
    }
}

impl From<WaveType> for WaveTypeProto {
    fn from(item: WaveType) -> Self {
        match item {
            WaveType::Sine => WaveTypeProto::Sine,
            WaveType::Square => WaveTypeProto::Square,
            WaveType::Saw => WaveTypeProto::Saw,
            WaveType::Triangle => WaveTypeProto::Triangle,
        }
    }
}
