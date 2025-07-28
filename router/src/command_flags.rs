use std::sync::{Condvar, Mutex, atomic::AtomicU8};


pub enum CommandFlag {
    AstarFrontierOrUpdatePosterior,
    AstarInOut,
    UpdatePosteriorResult,
    ProbaModelResult,
    Auto,
}

impl CommandFlag {
    pub fn get_level(&self) -> u8 {
        match self {
            CommandFlag::AstarFrontierOrUpdatePosterior => 0,
            CommandFlag::AstarInOut => 1,
            CommandFlag::UpdatePosteriorResult => 2,
            CommandFlag::ProbaModelResult => 3,
            CommandFlag::Auto => 4,
        }
    }
}
pub static TARGET_COMMAND_LEVEL: AtomicU8 = AtomicU8::new(0);