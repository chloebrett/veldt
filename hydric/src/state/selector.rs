pub enum Selector {
    Root,
    Track(/* track_index */ usize),
    Note(/* track_index */ usize, /* note_index */ usize),
    Effect(/* mixer_index */ usize, /* effect_index */ usize),
    Generator(/* generator_index */ usize),
}
