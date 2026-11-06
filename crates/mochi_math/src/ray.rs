/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{Float, Plane, Vec3, AABB};


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ray<T: Float = f32> {
	pub origin: Vec3<T>,
	pub direction: Vec3<T>,
}


impl<T: Float> Ray<T> {
	pub fn new(origin: Vec3<T>, direction: Vec3<T>) -> Self {
		Self {
			origin,
			direction: direction.normalize(),
		}
	}

	pub const fn new_unnormalized(origin: Vec3<T>, direction: Vec3<T>) -> Self {
		Self { origin, direction }
	}

	pub fn point_at(&self, t: T) -> Vec3<T> {
		self.origin + self.direction * t
	}

	pub fn intersects_plane(&self, plane: &Plane<T>) -> Option<T> {
		let denom = plane.normal.dot(&self.direction);
		let eps = T::from_f32(1e-6);

		if denom.abs() > eps {
			let t = -(plane.normal.dot(&self.origin) + plane.distance) / denom;
			if t >= T::ZERO {
				return Some(t);
			}
		}

		None
	}

	pub fn intersects_aabb(&self, aabb: &AABB<T>) -> Option<T> {
		let mut t_min = T::ZERO;
		let mut t_max = T::from_f32(f32::MAX);
		let eps = T::from_f32(1e-8);

		// X slab
		if self.direction.x.abs() > eps {
			let inv_d = T::ONE / self.direction.x;
			let mut t1 = (aabb.min.x - self.origin.x) * inv_d;
			let mut t2 = (aabb.max.x - self.origin.x) * inv_d;
			if t1 > t2 {
				core::mem::swap(&mut t1, &mut t2);
			}
			t_min = t_min.max(t1);
			t_max = t_max.min(t2);
			if t_min > t_max {
				return None;
			}
		} else if self.origin.x < aabb.min.x || self.origin.x > aabb.max.x {
			return None;
		}

		// Y slab
		if self.direction.y.abs() > eps {
			let inv_d = T::ONE / self.direction.y;
			let mut t1 = (aabb.min.y - self.origin.y) * inv_d;
			let mut t2 = (aabb.max.y - self.origin.y) * inv_d;
			if t1 > t2 {
				core::mem::swap(&mut t1, &mut t2);
			}
			t_min = t_min.max(t1);
			t_max = t_max.min(t2);
			if t_min > t_max {
				return None;
			}
		} else if self.origin.y < aabb.min.y || self.origin.y > aabb.max.y {
			return None;
		}

		// Z slab
		if self.direction.z.abs() > eps {
			let inv_d = T::ONE / self.direction.z;
			let mut t1 = (aabb.min.z - self.origin.z) * inv_d;
			let mut t2 = (aabb.max.z - self.origin.z) * inv_d;
			if t1 > t2 {
				core::mem::swap(&mut t1, &mut t2);
			}
			t_min = t_min.max(t1);
			t_max = t_max.min(t2);
			if t_min > t_max {
				return None;
			}
		} else if self.origin.z < aabb.min.z || self.origin.z > aabb.max.z {
			return None;
		}

		Some(t_min)
	}

	pub fn intersects_sphere(&self, center: Vec3<T>, radius: T) -> Option<T> {
		let oc = self.origin - center;
		let a = self.direction.dot(&self.direction);
		let b = T::TWO * oc.dot(&self.direction);
		let c = oc.dot(&oc) - radius * radius;
		let discriminant = b * b - (T::TWO + T::TWO) * a * c;

		if discriminant < T::ZERO {
			return None;
		}

		let sqrt_disc = discriminant.sqrt();
		let two_a = T::TWO * a;
		let t0 = (-b - sqrt_disc) / two_a;
		let t1 = (-b + sqrt_disc) / two_a;

		if t0 >= T::ZERO {
			Some(t0)
		} else if t1 >= T::ZERO {
			Some(t1)
		} else {
			None
		}
	}
}


impl<T: Float> Default for Ray<T> {
	fn default() -> Self {
		Self {
			origin: Vec3::ZERO,
			direction: Vec3::FORWARD,
		}
	}
}


impl<T: Float> From<(Vec3<T>, Vec3<T>)> for Ray<T> {
	fn from((origin, direction): (Vec3<T>, Vec3<T>)) -> Self {
		Self::new(origin, direction)
	}
}

impl<T: Float> From<[Vec3<T>; 2]> for Ray<T> {
	fn from(arr: [Vec3<T>; 2]) -> Self {
		Self::new(arr[0], arr[1])
	}
}
