/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashSet;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};


#[derive(Default, Debug, Clone)]
pub struct Input {
	keys_down: HashSet<u32>,
	keys_just_pressed: HashSet<u32>,
	keys_just_released: HashSet<u32>,

	mouse_buttons_down: HashSet<u8>,
	mouse_buttons_just_pressed: HashSet<u8>,
	mouse_buttons_just_released: HashSet<u8>,

	mouse_position: (f32, f32),
	last_mouse_position: Option<(f32, f32)>,
	mouse_delta: (f32, f32),
	mouse_wheel_delta: f32,
}

impl Input {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn is_key_down(&self, keycode: u32) -> bool {
		self.keys_down.contains(&keycode)
	}

	pub fn is_key_just_pressed(&self, keycode: u32) -> bool {
		self.keys_just_pressed.contains(&keycode)
	}

	pub fn is_key_just_released(&self, keycode: u32) -> bool {
		self.keys_just_released.contains(&keycode)
	}

	pub fn is_mouse_button_down(&self, button: u8) -> bool {
		self.mouse_buttons_down.contains(&button)
	}

	pub fn is_mouse_button_just_pressed(&self, button: u8) -> bool {
		self.mouse_buttons_just_pressed.contains(&button)
	}

	pub fn mouse_position(&self) -> (f32, f32) {
		self.mouse_position
	}

	pub fn mouse_delta(&self) -> (f32, f32) {
		self.mouse_delta
	}

	pub fn mouse_wheel_delta(&self) -> f32 {
		self.mouse_wheel_delta
	}

	pub fn reset_frame(&mut self) {
		self.keys_just_pressed.clear();
		self.keys_just_released.clear();
		self.mouse_buttons_just_pressed.clear();
		self.mouse_buttons_just_released.clear();
		self.mouse_delta = (0.0, 0.0);
		self.mouse_wheel_delta = 0.0;
	}

	pub fn handle_window_event(&mut self, event: &WindowEvent) {
		match event {
			WindowEvent::KeyboardInput {
				event:
					KeyEvent {
						physical_key: PhysicalKey::Code(code),
						state,
						..
					},
				..
			} => {
				let keycode = keycode_to_u32(*code);
				match state {
					ElementState::Pressed => {
						if !self.keys_down.contains(&keycode) {
							self.keys_just_pressed.insert(keycode);
							self.keys_down.insert(keycode);
						}
					}
					ElementState::Released => {
						self.keys_down.remove(&keycode);
						self.keys_just_released.insert(keycode);
					}
				}
			}
			WindowEvent::CursorMoved { position, .. } => {
				let current = (position.x as f32, position.y as f32);
				if let Some(last) = self.last_mouse_position {
					self.mouse_delta.0 += current.0 - last.0;
					self.mouse_delta.1 += current.1 - last.1;
				}
				self.last_mouse_position = Some(current);
				self.mouse_position = current;
			}
			WindowEvent::CursorLeft { .. } => {
				self.last_mouse_position = None;
			}
			WindowEvent::MouseInput { state, button, .. } => {
				let btn_id = match button {
					MouseButton::Left => 0,
					MouseButton::Right => 1,
					MouseButton::Middle => 2,
					MouseButton::Back => 3,
					MouseButton::Forward => 4,
					MouseButton::Other(id) => 10 + *id as u8,
				};
				match state {
					ElementState::Pressed => {
						if !self.mouse_buttons_down.contains(&btn_id) {
							self.mouse_buttons_just_pressed.insert(btn_id);
							self.mouse_buttons_down.insert(btn_id);
						}
					}
					ElementState::Released => {
						self.mouse_buttons_down.remove(&btn_id);
						self.mouse_buttons_just_released.insert(btn_id);
					}
				}
			}
			WindowEvent::MouseWheel { delta, .. } => {
				match delta {
					winit::event::MouseScrollDelta::LineDelta(_x, y) => {
						self.mouse_wheel_delta += *y;
					}
					winit::event::MouseScrollDelta::PixelDelta(pos) => {
						self.mouse_wheel_delta += (pos.y as f32) / 32.0;
					}
				}
			}
			_ => {}
		}
	}
}

pub fn keycode_to_u32(code: KeyCode) -> u32 {
	match code {
		KeyCode::KeyA => 65,
		KeyCode::KeyB => 66,
		KeyCode::KeyC => 67,
		KeyCode::KeyD => 68,
		KeyCode::KeyE => 69,
		KeyCode::KeyF => 70,
		KeyCode::KeyG => 71,
		KeyCode::KeyH => 72,
		KeyCode::KeyI => 73,
		KeyCode::KeyJ => 74,
		KeyCode::KeyK => 75,
		KeyCode::KeyL => 76,
		KeyCode::KeyM => 77,
		KeyCode::KeyN => 78,
		KeyCode::KeyO => 79,
		KeyCode::KeyP => 80,
		KeyCode::KeyQ => 81,
		KeyCode::KeyR => 82,
		KeyCode::KeyS => 83,
		KeyCode::KeyT => 84,
		KeyCode::KeyU => 85,
		KeyCode::KeyV => 86,
		KeyCode::KeyW => 87,
		KeyCode::KeyX => 88,
		KeyCode::KeyY => 89,
		KeyCode::KeyZ => 90,

		KeyCode::Digit0 => 48,
		KeyCode::Digit1 => 49,
		KeyCode::Digit2 => 50,
		KeyCode::Digit3 => 51,
		KeyCode::Digit4 => 52,
		KeyCode::Digit5 => 53,
		KeyCode::Digit6 => 54,
		KeyCode::Digit7 => 55,
		KeyCode::Digit8 => 56,
		KeyCode::Digit9 => 57,

		KeyCode::Space => 32,
		KeyCode::Escape => 27,
		KeyCode::Enter => 13,
		KeyCode::Tab => 9,
		KeyCode::Backspace => 8,
		KeyCode::ShiftLeft | KeyCode::ShiftRight => 16,
		KeyCode::ControlLeft | KeyCode::ControlRight => 17,
		KeyCode::AltLeft | KeyCode::AltRight => 18,

		KeyCode::ArrowUp => 38,
		KeyCode::ArrowDown => 40,
		KeyCode::ArrowLeft => 37,
		KeyCode::ArrowRight => 39,

		_ => 0,
	}
}
