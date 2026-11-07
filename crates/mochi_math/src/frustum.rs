/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{Float, Mat4x4, Plane, Vec3, AABB};


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Frustum<T: Float = f32> {
	pub planes: [Plane<T>; 6],
}

pub const FRUSTUM_LEFT: usize = 0;
pub const FRUSTUM_RIGHT: usize = 1;
pub const FRUSTUM_BOTTOM: usize = 2;
pub const FRUSTUM_TOP: usize = 3;
pub const FRUSTUM_NEAR: usize = 4;
pub const FRUSTUM_FAR: usize = 5;


impl<T: Float> Frustum<T> {
	pub const fn new(planes: [Plane<T>; 6]) -> Self {
		Self { planes }
	}


	pub fn from_view_projection(vp: &Mat4x4<T>) -> Self {
		let r0 = vp.rows[0];
		let r1 = vp.rows[1];
		let r2 = vp.rows[2];
		let r3 = vp.rows[3];

		let left = Plane::from_vec4(r3 + r0).normalize();
		let right = Plane::from_vec4(r3 - r0).normalize();
		let bottom = Plane::from_vec4(r3 + r1).normalize();
		let top = Plane::from_vec4(r3 - r1).normalize();
		let near = Plane::from_vec4(r2).normalize();
		let far = Plane::from_vec4(r3 - r2).normalize();

		Self {
			planes: [left, right, bottom, top, near, far],
		}
	}

	pub fn contains_point(&self, p: Vec3<T>) -> bool {
		for plane in &self.planes {
			if plane.distance_to_point(p) < T::ZERO {
				return false;
			}
		}
		true
	}

	pub fn intersects_sphere(&self, center: Vec3<T>, radius: T) -> bool {
		for plane in &self.planes {
			if plane.distance_to_point(center) < -radius {
				return false;
			}
		}
		true
	}

	pub fn intersects_aabb(&self, aabb: &AABB<T>) -> bool {
		for plane in &self.planes {
			// Find positive vertex (furthest in direction of normal)
			let px = if plane.normal.x >= T::ZERO { aabb.max.x } else { aabb.min.x };
			let py = if plane.normal.y >= T::ZERO { aabb.max.y } else { aabb.min.y };
			let pz = if plane.normal.z >= T::ZERO { aabb.max.z } else { aabb.min.z };

			if plane.distance_to_point(Vec3::new(px, py, pz)) < T::ZERO {
				return false;
			}
		}
		true
	}
}

impl<T: Float> Default for Frustum<T> {
	fn default() -> Self {
		Self {
			planes: [Plane::default(); 6],
		}
	}
}
