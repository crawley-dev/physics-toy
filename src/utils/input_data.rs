use std::time::{Duration, Instant};

use educe::Educe;
use winit::keyboard::KeyCode;

use crate::utils::{
    consts::{MOUSE_DRAG_THRESHOLD_PX, MOUSE_HOLD_THRESHOLD_MS},
    vec2::{Vec2, WindowSpace},
};

#[derive(Educe, Clone, Copy)]
#[educe(Debug)]
pub struct MouseInput {
    pub state: bool,
    pub time: Instant,
    pub pos: Vec2<f64, WindowSpace>,
}

#[derive(Educe, Clone, Copy)]
#[educe(Debug)]
pub struct InputData {
    pub mouse_pos: Vec2<f64, WindowSpace>,
    // this is used for holds, if mouse_down but not mouse_pressed
    // then it is held.
    pub mouse_down: bool,
    // this records the press event, holding its current state, the time of press and pos of press
    // additionally, this will operate on a cooldown, to prevent multiple presses (e.g. 3 frames << unavoidable by a human)
    pub mouse_pressed: MouseInput, // records an event's current state, with timestamp of press
    // this records the release event, holding its current state, the time of release and pos of release
    // this is currently (13/11) used for the gravity_sim angry birds particle fire!
    pub mouse_released: MouseInput, // records an event's current state, with timestamp of press

    // TODO(TOM): should keys_held have a cooldown?
    // both fields have a tap_cooldown, however "keys_tapped is reset each frame"
    #[educe(Debug(ignore))]
    pub keys_held: [bool; 256],
    #[educe(Debug(ignore))]
    pub keys_pressed: [bool; 256],
    #[educe(Debug(ignore))]
    pub tap_cooldowns: [Instant; 256],
}

impl InputData {
    pub const fn is_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed[key as usize]
    }
    pub const fn is_held(&self, key: KeyCode) -> bool {
        self.keys_held[key as usize]
    }

    pub fn is_mouse_pressed(&self) -> bool {
        self.mouse_pressed.state
    }

    pub fn is_mouse_down(&self) -> bool {
        self.mouse_down
    }

    // if the mouse is down and the cursor has moved more than +/- N pixels in either direction
    pub fn is_mouse_dragging(&self) -> bool {
        self.mouse_down && {
            let delta = self.mouse_pos - self.mouse_pressed.pos;
            delta.x.abs() >= MOUSE_DRAG_THRESHOLD_PX || delta.y.abs() >= MOUSE_DRAG_THRESHOLD_PX
        }
    }

    // if mouse is down and time since is greater than threshold.
    // TODO(TOM): this is a quite bad, only starts emitting true after user has been clicking for 250ms or more..
    pub fn is_mouse_held(&self) -> bool {
        self.mouse_down
            && self.mouse_pressed.time.elapsed() > Duration::from_millis(MOUSE_HOLD_THRESHOLD_MS)
    }

    pub fn was_mouse_dragging(&self) -> bool {
        self.mouse_released.state && {
            let delta = self.mouse_released.pos - self.mouse_pressed.pos;
            delta.x.abs() >= MOUSE_DRAG_THRESHOLD_PX || delta.y.abs() >= MOUSE_DRAG_THRESHOLD_PX
        }
    }

    // if mouse was released and time since is greater than threshold
    pub fn was_mouse_held(&self) -> bool {
        self.mouse_released.state
            && self.mouse_released.time - self.mouse_pressed.time
                > Duration::from_millis(MOUSE_HOLD_THRESHOLD_MS)
    }

    // if mouse released and time since is less than threshold
    pub fn was_mouse_pressed(&self) -> bool {
        self.mouse_released.state
            && self.mouse_released.time - self.mouse_pressed.time
                < Duration::from_millis(MOUSE_HOLD_THRESHOLD_MS)
    }
}
