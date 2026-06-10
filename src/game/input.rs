//! Estado de input — teclado e mouse.

use winit::event::{ElementState, MouseButton};
use winit::keyboard::KeyCode;

/// Estado do input capturado a cada frame.
#[derive(Debug, Default)]
pub struct InputState {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub run: bool,
    pub shoot: bool,
    pub interact: bool,
    pub shear: bool,
    pub build_mode: bool,
    pub place_block: bool,
    pub remove_block: bool,
    pub craft_fence: bool,
    pub release_herd: bool,
    pub cycle_weapon: bool,
    pub rotate_build: bool,
    pub level_up: bool,
    pub level_down: bool,
    pub ignite: bool,
    pub save_game: bool,
    pub tame: bool,
    pub hotbar_select: Option<usize>,
    pub mouse_delta: (f32, f32),
    pub cursor_grabbed: bool,
    pub toggle_grab: bool,
}

impl InputState {
    pub fn reset_frame(&mut self) {
        self.shoot = false;
        self.interact = false;
        self.shear = false;
        self.place_block = false;
        self.remove_block = false;
        self.craft_fence = false;
        self.release_herd = false;
        self.cycle_weapon = false;
        self.rotate_build = false;
        self.level_up = false;
        self.level_down = false;
        self.ignite = false;
        self.save_game = false;
        self.tame = false;
        self.hotbar_select = None;
        self.mouse_delta = (0.0, 0.0);
        self.toggle_grab = false;
    }

    pub fn on_key(&mut self, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::KeyW => self.forward = pressed,
            KeyCode::KeyS => self.backward = pressed,
            KeyCode::KeyA => self.left = pressed,
            KeyCode::KeyD => self.right = pressed,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => self.run = pressed,
            KeyCode::Escape if pressed => self.toggle_grab = true,
            KeyCode::KeyE if pressed => self.interact = true,
            KeyCode::KeyF if pressed => self.shear = true,
            KeyCode::KeyB if pressed => self.build_mode = !self.build_mode,
            KeyCode::KeyC if pressed => self.craft_fence = true,
            KeyCode::KeyQ if pressed => self.release_herd = true,
            KeyCode::Digit1 if pressed => self.hotbar_select = Some(0),
            KeyCode::Digit2 if pressed => self.hotbar_select = Some(1),
            KeyCode::Digit3 if pressed => self.hotbar_select = Some(2),
            KeyCode::Digit4 if pressed => self.hotbar_select = Some(3),
            KeyCode::Digit5 if pressed => self.hotbar_select = Some(4),
            KeyCode::Digit6 if pressed => self.hotbar_select = Some(5),
            KeyCode::Digit7 if pressed => self.hotbar_select = Some(6),
            KeyCode::Tab if pressed => self.cycle_weapon = true,
            KeyCode::KeyR if pressed => self.rotate_build = true,
            KeyCode::KeyX if pressed => self.level_up = true,
            KeyCode::KeyZ if pressed => self.level_down = true,
            KeyCode::KeyG if pressed => self.ignite = true,
            KeyCode::KeyP if pressed => self.save_game = true,
            KeyCode::KeyT if pressed => self.tame = true,
            _ => {}
        }
    }

    pub fn on_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        if state != ElementState::Pressed {
            return;
        }
        match button {
            MouseButton::Left => {
                if self.build_mode {
                    self.place_block = true;
                } else {
                    self.shoot = true;
                }
                if !self.cursor_grabbed {
                    self.cursor_grabbed = true;
                }
            }
            MouseButton::Right if self.build_mode => self.remove_block = true,
            MouseButton::Right if !self.build_mode => {
                self.shear = true;
            }
            _ => {}
        }
    }

    pub fn on_mouse_delta(&mut self, dx: f32, dy: f32) {
        if self.cursor_grabbed {
            self.mouse_delta.0 += dx;
            self.mouse_delta.1 += dy;
        }
    }
}
