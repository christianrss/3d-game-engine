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
    pub mouse_delta: (f32, f32),
    pub cursor_grabbed: bool,
    pub toggle_grab: bool,
}

impl InputState {
    pub fn reset_frame(&mut self) {
        self.shoot = false;
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
            _ => {}
        }
    }

    pub fn on_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        if button == MouseButton::Left && state == ElementState::Pressed {
            self.shoot = true;
            if !self.cursor_grabbed {
                self.cursor_grabbed = true;
            }
        }
    }

    pub fn on_mouse_delta(&mut self, dx: f32, dy: f32) {
        if self.cursor_grabbed {
            self.mouse_delta.0 += dx;
            self.mouse_delta.1 += dy;
        }
    }
}
