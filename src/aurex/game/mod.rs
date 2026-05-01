pub mod library;
pub mod blocks_and_bricks;
pub mod tech_demo;

#[derive(Clone, Copy, Debug, Default)]
pub struct InputState {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub accept: bool,
    pub cancel: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum AudioCue {
    #[default]
    None,
    SelectTrack(u8),
    LaunchRequest,
    Cancel,
}
