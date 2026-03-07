use crate::aurex::game::AudioCue;

#[derive(Clone, Copy, Debug)]
pub enum RuntimeEvent {
    Audio(AudioCue),
}

pub struct RuntimeEventQueue {
    events: Vec<RuntimeEvent>,
}

impl RuntimeEventQueue {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            events: Vec::with_capacity(cap),
        }
    }

    pub fn push(&mut self, event: RuntimeEvent) {
        self.events.push(event);
    }

    pub fn drain_to(&mut self, out: &mut Vec<RuntimeEvent>) {
        out.extend(self.events.drain(..));
    }
}
