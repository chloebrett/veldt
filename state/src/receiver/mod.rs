use crate::Action;

mod compressor_config;
mod delay_config;
mod effect_instance;
mod eq_config;
mod mod_delay_config;

pub trait ActionReceiver {
    fn apply(&mut self, action: &Action) -> Action;
}
