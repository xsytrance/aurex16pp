pub mod tech_demo;

#[derive(Clone, Copy, Debug, Default)]
pub struct InputState {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
}
