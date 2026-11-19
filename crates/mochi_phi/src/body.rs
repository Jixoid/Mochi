/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_math::{Quat, Vec3};


#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BodyHandle(pub u32);

impl BodyHandle {
	pub const INVALID: Self = Self(u32::MAX);

	#[inline]
	pub fn is_valid(self) -> bool {
		self != Self::INVALID
	}
}

impl Default for BodyHandle {
	fn default() -> Self {
		Self::INVALID
	}
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum BodyType {
	/// Unmoving body with infinite mass (floors, walls, terrain). Zero inverse mass.
	Static,
	/// Moving body controlled solely by script/code transforms; unaffected by physical forces or gravity.
	Kinematic,
	/// Fully simulated dynamic body obeying forces, impulses, gravity, collisions and constraints.
	#[default]
	Dynamic,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BodyDef {
	pub body_type: BodyType,
	pub mass: f32,
	pub linear_damping: f32,
	pub angular_damping: f32,
	pub gravity_scale: f32,
	pub can_sleep: bool,
	pub use_ccd: bool,
	pub position: Vec3<f32>,
	pub rotation: Quat<f32>,
	pub linear_velocity: Vec3<f32>,
	pub angular_velocity: Vec3<f32>,
	pub user_data: u64,
}

impl Default for BodyDef {
	fn default() -> Self {
		Self {
			body_type: BodyType::Dynamic,
			mass: 1.0,
			linear_damping: 0.05,
			angular_damping: 0.1,
			gravity_scale: 1.0,
			can_sleep: true,
			use_ccd: false,
			position: Vec3::ZERO,
			rotation: Quat::IDENTITY,
			linear_velocity: Vec3::ZERO,
			angular_velocity: Vec3::ZERO,
			user_data: 0,
		}
	}
}

impl BodyDef {
	pub fn dynamic() -> Self {
		Self {
			body_type: BodyType::Dynamic,
			..Default::default()
		}
	}

	pub fn static_body() -> Self {
		Self {
			body_type: BodyType::Static,
			mass: 0.0,
			can_sleep: true,
			..Default::default()
		}
	}

	pub fn kinematic() -> Self {
		Self {
			body_type: BodyType::Kinematic,
			mass: 0.0,
			..Default::default()
		}
	}

	pub fn with_position(mut self, pos: Vec3<f32>) -> Self {
		self.position = pos;
		self
	}

	pub fn with_rotation(mut self, rot: Quat<f32>) -> Self {
		self.rotation = rot;
		self
	}

	pub fn with_mass(mut self, mass: f32) -> Self {
		self.mass = mass;
		self
	}

	pub fn with_ccd(mut self, use_ccd: bool) -> Self {
		self.use_ccd = use_ccd;
		self
	}
}
