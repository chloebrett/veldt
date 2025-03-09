mod app;
mod audio_player;
mod audio_render;
mod audio_player_cpal;
mod init;
mod note_save;

use crate::init::init;

fn main() {
    init();
}
