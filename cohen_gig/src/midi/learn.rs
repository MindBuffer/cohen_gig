use super::MidiMessage;
use crate::midi::mapping::MidiTarget;
use std::time::Instant;

const LEARNED_FLASH_SECS: f64 = 0.5;

pub enum LearnState {
    Idle,
    Listening(MidiTarget),
    Learned(Instant),
}

impl LearnState {
    pub fn start(target: MidiTarget) -> Self {
        Self::Listening(target)
    }

    pub fn cancel() -> Self {
        Self::Idle
    }

    /// If currently listening, consume the message and return the (port, cc) to assign.
    pub fn receive(&mut self, msg: &MidiMessage) -> Option<(String, u8, MidiTarget)> {
        if let Self::Listening(target) = self {
            let result = (msg.port_name.clone(), msg.cc, *target);
            *self = Self::Learned(Instant::now());
            Some(result)
        } else {
            None
        }
    }

    pub fn update(&mut self) {
        if let Self::Learned(t) = self {
            if t.elapsed().as_secs_f64() >= LEARNED_FLASH_SECS {
                *self = Self::Idle;
            }
        }
    }

    pub fn is_listening(&self) -> bool {
        matches!(self, Self::Listening(_))
    }

    pub fn is_listening_for(&self, target: MidiTarget) -> bool {
        matches!(self, Self::Listening(t) if *t == target)
    }

    #[allow(dead_code)]
    pub fn is_learned(&self) -> bool {
        matches!(self, Self::Learned(_))
    }
}

impl Default for LearnState {
    fn default() -> Self {
        Self::Idle
    }
}
