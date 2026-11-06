/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{Float, Mat4x4, Number, Vec3};


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AABB<T: Number = f32> {
	pub min: Vec3<T>,
	pub max: Vec3<T>,
}


impl<T: Number> AABB<T> {
	pub const ZERO: Self = Self {min: Vec3::ZERO, max: Vec3::ZERO};


	pub const fn new(min: Vec3<T>, max: Vec3<T>) -> Self {
		Self { min, max }
	}

	pub fn from_center_half_extents(center: Vec3<T>, half: Vec3<T>) -> Self {
		Self {
			min: center - half,
			max: center + half,
		}
	}

	pub fn from_points(points: &[Vec3<T>]) -> Self {
		if points.is_empty() {
			return Self::ZERO;
		}

		let mut min = points[0];
		let mut max = points[0];

		for &p in &points[1..] {
			min = min.min(&p);
			max = max.max(&p);
		}

		Self { min, max }
	}

	pub fn center(&self) -> Vec3<T> {
		let half = T::ONE / (T::ONE + T::ONE);
		(self.min + self.max) * half
	}

	pub fn size(&self) -> Vec3<T> {
		self.max - self.min
	}

	pub fn half_extents(&self) -> Vec3<T> {
		let half = T::ONE / (T::ONE + T::ONE);
		self.size() * half
	}

	pub fn contains_point(&self, p: Vec3<T>) -> bool {
		p.x >= self.min.x && p.x <= self.max.x
			&& p.y >= self.min.y && p.y <= self.max.y
			&& p.z >= self.min.z && p.z <= self.max.z
	}

	pub fn intersects(&self, other: &Self) -> bool {
		self.min.x <= other.max.x && self.max.x >= other.min.x
			&& self.min.y <= other.max.y && self.max.y >= other.min.y
			&& self.min.z <= other.max.z && self.max.z >= other.min.z
	}

	pub fn expand(&mut self, p: Vec3<T>) {
		self.min = self.min.min(&p);
		self.max = self.max.max(&p);
	}

	pub fn encapsulate(&mut self, other: &Self) {
		self.min = self.min.min(&other.min);
		self.max = self.max.max(&other.max);
	}
}

impl<T: Float> AABB<T> {
	pub fn corners(&self) -> [Vec3<T>; 8] {
		[
			Vec3::new(self.min.x, self.min.y, self.min.z),
			Vec3::new(self.max.x, self.min.y, self.min.z),
			Vec3::new(self.min.x, self.max.y, self.min.z),
			Vec3::new(self.max.x, self.max.y, self.min.z),
			Vec3::new(self.min.x, self.min.y, self.max.z),
			Vec3::new(self.max.x, self.min.y, self.max.z),
			Vec3::new(self.min.x, self.max.y, self.max.z),
			Vec3::new(self.max.x, self.max.y, self.max.z),
		]
	}

	pub fn transformed(&self, m: &Mat4x4<T>) -> Self {
		let corners = self.corners();
		let mut transformed_corners = [Vec3::ZERO; 8];

		for i in 0..8 {
			transformed_corners[i] = *m * corners[i];
		}

		Self::from_points(&transformed_corners)
	}
}


impl<T: Number> Default for AABB<T> {
	fn default() -> Self {
		Self::ZERO
	}
}


impl<T: Number> From<(Vec3<T>, Vec3<T>)> for AABB<T> {
	fn from((min, max): (Vec3<T>, Vec3<T>)) -> Self {
		Self::new(min, max)
	}
}

impl<T: Number> From<[Vec3<T>; 2]> for AABB<T> {
	fn from(arr: [Vec3<T>; 2]) -> Self {
		Self::new(arr[0], arr[1])
	}
}
