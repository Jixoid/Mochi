/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{Float, Vec3, Vec4};


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Plane<T: Float = f32> {
	pub normal: Vec3<T>,
	pub distance: T,
}


impl<T: Float> Plane<T> {
	pub const fn new(normal: Vec3<T>, distance: T) -> Self {
		Self { normal, distance }
	}


	pub fn from_point_normal(point: Vec3<T>, normal: Vec3<T>) -> Self {
		let n = normal.normalize();
		let distance = -n.dot(&point);
		Self { normal: n, distance }
	}

	pub fn from_coplanar_points(a: Vec3<T>, b: Vec3<T>, c: Vec3<T>) -> Self {
		let normal = (b - a).cross(&(c - a)).normalize();
		Self::from_point_normal(a, normal)
	}

	pub fn from_vec4(v: Vec4<T>) -> Self {
		Self {
			normal: Vec3::new(v.x, v.y, v.z),
			distance: v.w,
		}
	}

	pub fn distance_to_point(&self, p: Vec3<T>) -> T {
		self.normal.dot(&p) + self.distance
	}

	pub fn normalize(&self) -> Self {
		let len = self.normal.length();
		if len > T::ZERO {
			let inv_len = T::ONE / len;
			Self {
				normal: self.normal * inv_len,
				distance: self.distance * inv_len,
			}
		} else {
			*self
		}
	}

	
	pub fn dot_point(&self, p: Vec3<T>) -> T {
		self.distance_to_point(p)
	}

	pub fn dot_vector(&self, v: Vec3<T>) -> T {
		self.normal.dot(&v)
	}
}

impl<T: Float> Default for Plane<T> {
	fn default() -> Self {
		Self {
			normal: Vec3::UP,
			distance: T::ZERO,
		}
	}
}


impl<T: Float> From<Vec4<T>> for Plane<T> {
	fn from(v: Vec4<T>) -> Self {
		Self::from_vec4(v)
	}
}

impl<T: Float> From<Plane<T>> for Vec4<T> {
	fn from(p: Plane<T>) -> Self {
		Vec4::new(p.normal.x, p.normal.y, p.normal.z, p.distance)
	}
}

impl<T: Float> From<(Vec3<T>, T)> for Plane<T> {
	fn from((normal, distance): (Vec3<T>, T)) -> Self {
		Self::new(normal, distance)
	}
}
