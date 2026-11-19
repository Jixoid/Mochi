/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_math::Vec3;
use crate::body::BodyHandle;


#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JointHandle(pub u32);

impl JointHandle {
	pub const INVALID: Self = Self(u32::MAX);

	#[inline]
	pub fn is_valid(self) -> bool {
		self != Self::INVALID
	}
}

impl Default for JointHandle {
	fn default() -> Self {
		Self::INVALID
	}
}


#[derive(Clone, Debug, PartialEq)]
pub struct DistanceJointDef {
	pub body_a: BodyHandle,
	pub body_b: BodyHandle,
	pub local_anchor_a: Vec3<f32>,
	pub local_anchor_b: Vec3<f32>,
	pub target_distance: f32,
	pub stiffness: f32,
	pub damping: f32,
}

impl Default for DistanceJointDef {
	fn default() -> Self {
		Self {
			body_a: BodyHandle::INVALID,
			body_b: BodyHandle::INVALID,
			local_anchor_a: Vec3::ZERO,
			local_anchor_b: Vec3::ZERO,
			target_distance: 1.0,
			stiffness: 1.0,
			damping: 0.1,
		}
	}
}


#[derive(Clone, Debug, PartialEq)]
pub struct RevoluteJointDef {
	pub body_a: BodyHandle,
	pub body_b: BodyHandle,
	pub local_anchor_a: Vec3<f32>,
	pub local_anchor_b: Vec3<f32>,
	pub local_axis_a: Vec3<f32>,
	pub local_axis_b: Vec3<f32>,
	pub enable_limits: bool,
	pub lower_angle_rad: f32,
	pub upper_angle_rad: f32,
}

impl Default for RevoluteJointDef {
	fn default() -> Self {
		Self {
			body_a: BodyHandle::INVALID,
			body_b: BodyHandle::INVALID,
			local_anchor_a: Vec3::ZERO,
			local_anchor_b: Vec3::ZERO,
			local_axis_a: Vec3::new(0.0, 1.0, 0.0),
			local_axis_b: Vec3::new(0.0, 1.0, 0.0),
			enable_limits: false,
			lower_angle_rad: 0.0,
			upper_angle_rad: 0.0,
		}
	}
}


#[derive(Clone, Debug, PartialEq)]
pub struct FixedJointDef {
	pub body_a: BodyHandle,
	pub body_b: BodyHandle,
	pub local_anchor_a: Vec3<f32>,
	pub local_anchor_b: Vec3<f32>,
}
