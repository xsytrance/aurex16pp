pub mod library;

#[derive(Clone, Copy, Debug, Default)]
pub struct InputState {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum AudioCue {
    #[default]
    None,
    SelectTrack(u8),
}
