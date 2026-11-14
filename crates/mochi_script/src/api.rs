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
use mochi_input::Input;
use mochi_scene::{PointLightComponent, SpotLightComponent, TransformComponent};
use mochi_rhi::MsaaSamples;
use mochi_rendering::AntiAliasingMode;


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


fn with_transform_mut<F: FnOnce(&mut TransformComponent)>(entity_id: u64, f: F) {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &mut *ctx.world.as_ptr() };
			if let Ok(mut transform) = world.get::<&mut TransformComponent>(entity) {
				f(&mut transform);
			}
		}
	});
}

fn with_transform<F: FnOnce(&TransformComponent)>(entity_id: u64, f: F) {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &*ctx.world.as_ptr() };
			if let Ok(transform) = world.get::<&TransformComponent>(entity) {
				f(&transform);
			}
		}
	});
}


#[unsafe(no_mangle)]
pub extern "C" fn mochi_get_position(entity_id: u64, out_pos: *mut CVec3) {
	if out_pos.is_null() {
		return;
	}
	with_transform(entity_id, |t| unsafe {
		(*out_pos).x = t.translation.x;
		(*out_pos).y = t.translation.y;
		(*out_pos).z = t.translation.z;
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_set_position(entity_id: u64, in_pos: *const CVec3) {
	if in_pos.is_null() {
		return;
	}
	let pos = unsafe { *in_pos };
	with_transform_mut(entity_id, |t| {
		t.translation = mochi_math::Vec3::new(pos.x, pos.y, pos.z);
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_rotate(entity_id: u64, dx: f32, dy: f32, dz: f32) {
	with_transform_mut(entity_id, |t| {
		let rx = mochi_math::Quat::from_axis_angle(
			mochi_math::Vec3::new(1.0, 0.0, 0.0),
			mochi_math::Rad::new(dx),
		);
		let ry = mochi_math::Quat::from_axis_angle(
			mochi_math::Vec3::new(0.0, 1.0, 0.0),
			mochi_math::Rad::new(dy),
		);
		let rz = mochi_math::Quat::from_axis_angle(
			mochi_math::Vec3::new(0.0, 0.0, 1.0),
			mochi_math::Rad::new(dz),
		);
		let delta = (ry * rx * rz).normalize();
		t.rotation = (t.rotation * delta).normalize();
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_move_relative(
	entity_id: u64,
	forward_delta: f32,
	right_delta: f32,
	up_delta: f32,
) {
	with_transform_mut(entity_id, |t| {
		let forward = t.rotation * mochi_math::Vec3::new(0.0, 0.0, -1.0);
		let right = t.rotation * mochi_math::Vec3::new(1.0, 0.0, 0.0);
		let up = mochi_math::Vec3::new(0.0, 1.0, 0.0);
		t.translation += forward * forward_delta + right * right_delta + up * up_delta;
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_camera_look(entity_id: u64, pitch_delta: f32, yaw_delta: f32) {
	with_transform_mut(entity_id, |t| {
		let yaw_rot = mochi_math::Quat::from_axis_angle(
			mochi_math::Vec3::new(0.0, 1.0, 0.0),
			mochi_math::Rad::new(yaw_delta),
		);
		let r1 = (yaw_rot * t.rotation).normalize();
		let local_right = r1 * mochi_math::Vec3::new(1.0, 0.0, 0.0);
		let pitch_rot =
			mochi_math::Quat::from_axis_angle(local_right, mochi_math::Rad::new(pitch_delta));
		t.rotation = (pitch_rot * r1).normalize();
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_get_scale(entity_id: u64, out_scale: *mut CVec3) {
	if out_scale.is_null() {
		return;
	}
	with_transform(entity_id, |t| unsafe {
		(*out_scale).x = t.scale.x;
		(*out_scale).y = t.scale.y;
		(*out_scale).z = t.scale.z;
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_set_scale(entity_id: u64, in_scale: *const CVec3) {
	if in_scale.is_null() {
		return;
	}
	let s = unsafe { *in_scale };
	with_transform_mut(entity_id, |t| {
		t.scale = mochi_math::Vec3::new(s.x, s.y, s.z);
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
				unsafe {
					*out_dx = dx;
				}
			}
			if !out_dy.is_null() {
				unsafe {
					*out_dy = dy;
				}
			}
		}
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_get_mouse_wheel() -> f32 {
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let input = unsafe { ctx.input.as_ref() };
			input.mouse_wheel_delta()
		} else {
			0.0
		}
	})
}

static CAMERA_SPEED_BITS: std::sync::atomic::AtomicU32 =
	std::sync::atomic::AtomicU32::new(5.5f32.to_bits());

#[unsafe(no_mangle)]
pub extern "C" fn mochi_get_camera_speed() -> f32 {
	f32::from_bits(CAMERA_SPEED_BITS.load(std::sync::atomic::Ordering::Relaxed))
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_set_camera_speed(speed: f32) {
	CAMERA_SPEED_BITS.store(
		speed.clamp(0.2, 100.0).to_bits(),
		std::sync::atomic::Ordering::Relaxed,
	);
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_get_time() -> f32 {
	CURRENT_CONTEXT.with(|ctx| ctx.borrow().as_ref().map(|c| c.time).unwrap_or(0.0))
}

static REQUESTED_MSAA: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static CURRENT_MSAA: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

pub fn take_requested_msaa() -> Option<MsaaSamples> {
	let val = REQUESTED_MSAA.swap(0, std::sync::atomic::Ordering::Relaxed);
	if val == 0 {
		None
	} else {
		Some(MsaaSamples::from_u32(val))
	}
}

pub fn set_current_msaa(samples: MsaaSamples) {
	CURRENT_MSAA.store(samples.count(), std::sync::atomic::Ordering::Relaxed);
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_set_msaa(samples: i32) {
	REQUESTED_MSAA.store(
		(samples.max(1)) as u32,
		std::sync::atomic::Ordering::Relaxed,
	);
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_get_msaa() -> i32 {
	CURRENT_MSAA.load(std::sync::atomic::Ordering::Relaxed) as i32
}

static REQUESTED_AA_MODE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(-1);
static CURRENT_AA_MODE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

pub fn take_requested_aa_mode() -> Option<AntiAliasingMode> {
	let val = REQUESTED_AA_MODE.swap(-1, std::sync::atomic::Ordering::Relaxed);
	if val < 0 {
		None
	} else {
		Some(AntiAliasingMode::from_i32(val))
	}
}

pub fn set_current_aa_mode(mode: AntiAliasingMode) {
	CURRENT_AA_MODE.store(mode.to_i32(), std::sync::atomic::Ordering::Relaxed);
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_set_aa_mode(mode: i32) {
	REQUESTED_AA_MODE.store(mode, std::sync::atomic::Ordering::Relaxed);
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_get_aa_mode() -> i32 {
	CURRENT_AA_MODE.load(std::sync::atomic::Ordering::Relaxed)
}

static REQUESTED_CURSOR_CAPTURE: std::sync::atomic::AtomicI32 =
	std::sync::atomic::AtomicI32::new(-1);
static CURRENT_CURSOR_CAPTURE: std::sync::atomic::AtomicBool =
	std::sync::atomic::AtomicBool::new(true);

pub fn take_requested_cursor_capture() -> Option<bool> {
	let val = REQUESTED_CURSOR_CAPTURE.swap(-1, std::sync::atomic::Ordering::Relaxed);
	if val < 0 { None } else { Some(val != 0) }
}

pub fn set_current_cursor_captured(captured: bool) {
	CURRENT_CURSOR_CAPTURE.store(captured, std::sync::atomic::Ordering::Relaxed);
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_set_cursor_captured(captured: bool) {
	REQUESTED_CURSOR_CAPTURE.store(
		if captured { 1 } else { 0 },
		std::sync::atomic::Ordering::Relaxed,
	);
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_is_cursor_captured() -> bool {
	CURRENT_CURSOR_CAPTURE.load(std::sync::atomic::Ordering::Relaxed)
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_toggle_cursor_capture() {
	let current = CURRENT_CURSOR_CAPTURE.load(std::sync::atomic::Ordering::Relaxed);
	REQUESTED_CURSOR_CAPTURE.store(
		if !current { 1 } else { 0 },
		std::sync::atomic::Ordering::Relaxed,
	);
}

// Spot Light Component API

#[unsafe(no_mangle)]
pub extern "C" fn mochi_has_spot_light(entity_id: u64) -> bool {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return false,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &*ctx.world.as_ptr() };
			world.get::<&SpotLightComponent>(entity).is_ok()
		} else {
			false
		}
	})
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_get_spot_light_intensity(entity_id: u64) -> f32 {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return 0.0,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &*ctx.world.as_ptr() };
			if let Ok(light) = world.get::<&SpotLightComponent>(entity) {
				return light.intensity;
			}
		}
		0.0
	})
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_set_spot_light_intensity(entity_id: u64, intensity: f32) {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &mut *ctx.world.as_ptr() };
			if let Ok(mut light) = world.get::<&mut SpotLightComponent>(entity) {
				light.intensity = intensity;
			}
		}
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_get_spot_light_radius(entity_id: u64) -> f32 {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return 0.0,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &*ctx.world.as_ptr() };
			if let Ok(light) = world.get::<&SpotLightComponent>(entity) {
				return light.radius;
			}
		}
		0.0
	})
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_set_spot_light_radius(entity_id: u64, radius: f32) {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &mut *ctx.world.as_ptr() };
			if let Ok(mut light) = world.get::<&mut SpotLightComponent>(entity) {
				light.radius = radius;
			}
		}
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_get_spot_light_cone(
	entity_id: u64,
	out_inner_deg: *mut f32,
	out_outer_deg: *mut f32,
) {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &*ctx.world.as_ptr() };
			if let Ok(light) = world.get::<&SpotLightComponent>(entity) {
				if !out_inner_deg.is_null() {
					unsafe { *out_inner_deg = light.inner_cone_angle_deg };
				}
				if !out_outer_deg.is_null() {
					unsafe { *out_outer_deg = light.outer_cone_angle_deg };
				}
			}
		}
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_set_spot_light_cone(entity_id: u64, inner_deg: f32, outer_deg: f32) {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &mut *ctx.world.as_ptr() };
			if let Ok(mut light) = world.get::<&mut SpotLightComponent>(entity) {
				light.inner_cone_angle_deg = inner_deg;
				light.outer_cone_angle_deg = outer_deg;
			}
		}
	});
}

// Point Light Component API

#[unsafe(no_mangle)]
pub extern "C" fn mochi_has_point_light(entity_id: u64) -> bool {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return false,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &*ctx.world.as_ptr() };
			world.get::<&PointLightComponent>(entity).is_ok()
		} else {
			false
		}
	})
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_get_point_light_intensity(entity_id: u64) -> f32 {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return 0.0,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &*ctx.world.as_ptr() };
			if let Ok(light) = world.get::<&PointLightComponent>(entity) {
				return light.intensity;
			}
		}
		0.0
	})
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_set_point_light_intensity(entity_id: u64, intensity: f32) {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &mut *ctx.world.as_ptr() };
			if let Ok(mut light) = world.get::<&mut PointLightComponent>(entity) {
				light.intensity = intensity;
			}
		}
	});
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_get_point_light_radius(entity_id: u64) -> f32 {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return 0.0,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &*ctx.world.as_ptr() };
			if let Ok(light) = world.get::<&PointLightComponent>(entity) {
				return light.radius;
			}
		}
		0.0
	})
}

#[unsafe(no_mangle)]
pub extern "C" fn mochi_set_point_light_radius(entity_id: u64, radius: f32) {
	let entity = match hecs::Entity::from_bits(entity_id) {
		Some(e) => e,
		None => return,
	};
	CURRENT_CONTEXT.with(|ctx| {
		if let Some(ctx) = ctx.borrow().as_ref() {
			let world = unsafe { &mut *ctx.world.as_ptr() };
			if let Ok(mut light) = world.get::<&mut PointLightComponent>(entity) {
				light.radius = radius;
			}
		}
	});
}
