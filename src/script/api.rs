/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/

#![allow(clippy::not_unsafe_ptr_arg_deref)]


use std::cell::RefCell;
use std::ptr::NonNull;
use crate::core::input::Input;
use crate::scene::TransformComponent;


#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct CVec3 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

pub struct ScriptContext {
	pub world: NonNull<hecs::World>,
	pub input: NonNull<Input>,
	pub time: f32,
}

thread_local! {
	static CURRENT_CONTEXT: RefCell<Option<ScriptContext>> = const { RefCell::new(None) };
}

pub struct ContextGuard;

impl ContextGuard {
	pub fn new(world: &mut hecs::World, input: &Input, time: f32) -> Self {
		CURRENT_CONTEXT.with(|ctx| {
			*ctx.borrow_mut() = Some(ScriptContext {
				world: NonNull::from(world),
				input: NonNull::from(input),
				time,
			});
		});
		Self
	}
}

impl Drop for ContextGuard {
	fn drop(&mut self) {
		CURRENT_CONTEXT.with(|ctx| {
			*ctx.borrow_mut() = None;
		});
	}
}


#[unsafe(no_mangle)]
pub extern "C" fn mochi_get_position(entity_id: u64, out_pos: *mut CVec3) {
	if out_pos.is_null() {
		return;
	}
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &*ctx.world.as_ptr() };
			if let Ok(transform) = world.get::<&TransformComponent>(entity) {
				unsafe {
					(*out_pos).x = transform.translation.x;
					(*out_pos).y = transform.translation.y;
					(*out_pos).z = transform.translation.z;
				}
			}
		}
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_set_position(entity_id: u64, in_pos: *const CVec3) {
	if in_pos.is_null() {
		return;
	}
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &mut *ctx.world.as_ptr() };
			if let Ok(mut transform) = world.get::<&mut TransformComponent>(entity) {
				let pos = unsafe { *in_pos };
				transform.translation = mochi_math::Vec3::new(pos.x, pos.y, pos.z);
			}
		}
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_rotate(entity_id: u64, dx: f32, dy: f32, dz: f32) {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &mut *ctx.world.as_ptr() };
			if let Ok(mut transform) = world.get::<&mut TransformComponent>(entity) {
				let rx = mochi_math::Quat::from_axis_angle(mochi_math::Vec3::new(1.0, 0.0, 0.0), mochi_math::Rad::new(dx));
				let ry = mochi_math::Quat::from_axis_angle(mochi_math::Vec3::new(0.0, 1.0, 0.0), mochi_math::Rad::new(dy));
				let rz = mochi_math::Quat::from_axis_angle(mochi_math::Vec3::new(0.0, 0.0, 1.0), mochi_math::Rad::new(dz));
				let delta = (ry * rx * rz).normalize();
				transform.rotation = (transform.rotation * delta).normalize();
			}
		}
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_move_relative(entity_id: u64, forward_delta: f32, right_delta: f32, up_delta: f32) {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &mut *ctx.world.as_ptr() };
			if let Ok(mut transform) = world.get::<&mut TransformComponent>(entity) {
				let forward = transform.rotation * mochi_math::Vec3::new(0.0, 0.0, -1.0);
				let right = transform.rotation * mochi_math::Vec3::new(1.0, 0.0, 0.0);
				let up = mochi_math::Vec3::new(0.0, 1.0, 0.0);

				transform.translation += forward * forward_delta + right * right_delta + up * up_delta;
			}
		}
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_camera_look(entity_id: u64, pitch_delta: f32, yaw_delta: f32) {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &mut *ctx.world.as_ptr() };
			if let Ok(mut transform) = world.get::<&mut TransformComponent>(entity) {
				let yaw_rot = mochi_math::Quat::from_axis_angle(
					mochi_math::Vec3::new(0.0, 1.0, 0.0),
					mochi_math::Rad::new(yaw_delta),
				);
				let r1 = (yaw_rot * transform.rotation).normalize();
				let local_right = r1 * mochi_math::Vec3::new(1.0, 0.0, 0.0);
				let pitch_rot = mochi_math::Quat::from_axis_angle(
					local_right,
					mochi_math::Rad::new(pitch_delta),
				);

				transform.rotation = (pitch_rot * r1).normalize();
			}
		}
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_get_scale(entity_id: u64, out_scale: *mut CVec3) {
	if out_scale.is_null() {
		return;
	}
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &*ctx.world.as_ptr() };
			if let Ok(transform) = world.get::<&TransformComponent>(entity) {
				unsafe {
					(*out_scale).x = transform.scale.x;
					(*out_scale).y = transform.scale.y;
					(*out_scale).z = transform.scale.z;
				}
			}
		}
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_set_scale(entity_id: u64, in_scale: *const CVec3) {
	if in_scale.is_null() {
		return;
	}
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &mut *ctx.world.as_ptr() };
			if let Ok(mut transform) = world.get::<&mut TransformComponent>(entity) {
				let s = unsafe { *in_scale };
				transform.scale = mochi_math::Vec3::new(s.x, s.y, s.z);
			}
		}
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_is_key_down(keycode: i32) -> bool {
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let input = unsafe { ctx.input.as_ref() };
			input.is_key_down(keycode as u32)
		} else {
			false
		}
	})
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_is_key_just_pressed(keycode: i32) -> bool {
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let input = unsafe { ctx.input.as_ref() };
			input.is_key_just_pressed(keycode as u32)
		} else {
			false
		}
	})
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_is_mouse_button_down(button: i32) -> bool {
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let input = unsafe { ctx.input.as_ref() };
			input.is_mouse_button_down(button as u8)
		} else {
			false
		}
	})
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_get_mouse_delta(out_dx: *mut f32, out_dy: *mut f32) {
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let input = unsafe { ctx.input.as_ref() };
			let (dx, dy) = input.mouse_delta();
			if !out_dx.is_null() {
				unsafe { *out_dx = dx; }
			}
			if !out_dy.is_null() {
				unsafe { *out_dy = dy; }
			}
		}
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_get_time() -> f32 {
	CURRENT_CONTEXT.with(|ctx| {
		ctx.borrow().as_ref().map(|c| c.time).unwrap_or(0.0)
	})
}
