use crate::aurex::game::AudioCue;

#[derive(Clone, Copy, Debug)]
pub enum RuntimeEvent {
    Audio(AudioCue),
}
