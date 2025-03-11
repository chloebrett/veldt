mod app;
mod audio_player;
mod audio_render;
mod envelope_control;
mod init;
mod note_save;

use crate::init::init;
pub use envelope_control::envelope_control;

fn main() {
    init();
}
