use crate::aurex::game::InputState;
use sdl2::EventPump;
use sdl2::controller::{Axis, Button, GameController};
use sdl2::keyboard::Scancode;

pub struct PolledInput {
    pub quit_requested: bool,
    pub start_pressed: bool,
    pub gameplay: InputState,
}

pub fn poll_input(
    pump: &EventPump,
    controller: Option<&GameController>,
    game_active: bool,
) -> PolledInput {
    let kb = pump.keyboard_state();

    let quit_requested = kb.is_scancode_pressed(Scancode::Escape);

    let start_key_pressed = kb.is_scancode_pressed(Scancode::Return)
        || kb.is_scancode_pressed(Scancode::Space)
        || kb.is_scancode_pressed(Scancode::LShift)
        || kb.is_scancode_pressed(Scancode::RShift)
        || kb.is_scancode_pressed(Scancode::LCtrl)
        || kb.is_scancode_pressed(Scancode::RCtrl)
        || kb.is_scancode_pressed(Scancode::Tab)
        || kb.is_scancode_pressed(Scancode::Up)
        || kb.is_scancode_pressed(Scancode::Down)
        || kb.is_scancode_pressed(Scancode::Left)
        || kb.is_scancode_pressed(Scancode::Right)
        || kb.is_scancode_pressed(Scancode::W)
        || kb.is_scancode_pressed(Scancode::A)
        || kb.is_scancode_pressed(Scancode::S)
        || kb.is_scancode_pressed(Scancode::D)
        || kb.is_scancode_pressed(Scancode::Z)
        || kb.is_scancode_pressed(Scancode::X);

    let mut pad_left = false;
    let mut pad_right = false;
    let mut pad_up = false;
    let mut pad_down = false;

    let mut pad_start = false;
    if let Some(c) = controller {
        let lx = c.axis(Axis::LeftX);
        let ly = c.axis(Axis::LeftY);
        let rx = c.axis(Axis::RightX);
        let ry = c.axis(Axis::RightY);
        let lt = c.axis(Axis::TriggerLeft);
        let rt = c.axis(Axis::TriggerRight);

        // XInput-friendly deadzones for analog movement.
        pad_left = lx < -10_000 || rx < -12_000 || c.button(Button::DPadLeft);
        pad_right = lx > 10_000 || rx > 12_000 || c.button(Button::DPadRight);
        pad_up = ly < -10_000 || ry < -12_000 || c.button(Button::DPadUp);
        pad_down = ly > 10_000 || ry > 12_000 || c.button(Button::DPadDown);

        pad_start = pad_left
            || pad_right
            || pad_up
            || pad_down
            || lt > 8_000
            || rt > 8_000
            || c.button(Button::A)
            || c.button(Button::B)
            || c.button(Button::X)
            || c.button(Button::Y)
            || c.button(Button::Back)
            || c.button(Button::Guide)
            || c.button(Button::Start)
            || c.button(Button::LeftShoulder)
            || c.button(Button::RightShoulder)
            || c.button(Button::LeftStick)
            || c.button(Button::RightStick);
    }

    let gameplay = if game_active {
        InputState {
            left: kb.is_scancode_pressed(Scancode::Left)
                || kb.is_scancode_pressed(Scancode::A)
                || pad_left,
            right: kb.is_scancode_pressed(Scancode::Right)
                || kb.is_scancode_pressed(Scancode::D)
                || pad_right,
            up: kb.is_scancode_pressed(Scancode::Up)
                || kb.is_scancode_pressed(Scancode::W)
                || pad_up,
            down: kb.is_scancode_pressed(Scancode::Down)
                || kb.is_scancode_pressed(Scancode::S)
                || pad_down,
        }
    } else {
        InputState::default()
    };

    PolledInput {
        quit_requested,
        start_pressed: start_key_pressed || pad_start,
        gameplay,
    }
}
