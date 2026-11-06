/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use core::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign};
use crate::{Number, Vec2};


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3<T: Number = f32> {
	pub x: T,
	pub y: T,
	pub z: T,
}


impl<T: Number + const Neg<Output = T>> Vec3<T> {
	pub const UP: Self = Self{x: T::ZERO, y: T::ONE, z: T::ZERO};
	pub const DOWN: Self = Self{x: T::ZERO, y: -T::ONE, z: T::ZERO};
	
	pub const LEFT: Self = Self{x: -T::ONE, y: T::ZERO, z: T::ZERO};
	pub const RIGHT: Self = Self{x: T::ONE, y: T::ZERO, z: T::ZERO};

	pub const FORWARD: Self = Self{x: T::ZERO, y: T::ZERO, z: -T::ONE};
	pub const BACK: Self = Self{x: T::ZERO, y: T::ZERO, z: T::ONE};
}

impl<T: Number> Vec3<T> {
	pub const ZERO: Self = Self{x: T::ZERO, y: T::ZERO, z: T::ZERO};
	pub const ONE: Self = Self{x: T::ONE, y: T::ONE, z: T::ONE};


	pub const fn new(x: T, y: T, z: T) -> Self {
		Self { x, y, z }
	}

	pub const fn new_with(x: T, y: T, z: T) -> Self {
		Self::new(x, y, z)
	}

	pub const fn new_cast(v: T) -> Self {
		Self { x: v, y: v, z: v }
	}


	pub fn dot(&self, it: &Self) -> T {
		(self.x * it.x) + (self.y * it.y) + (self.z * it.z)
	}

	pub fn dist_sq(&self, it: &Self) -> T {
		let dx = self.x - it.x;
		let dy = self.y - it.y;
		let dz = self.z - it.z;
		(dx * dx) + (dy * dy) + (dz * dz)
	}

	pub fn dist(&self, it: &Self) -> T {
		self.dist_sq(it).sqrt()
	}

	pub fn length(&self) -> T {
		self.dot(self).sqrt()
	}

	pub fn cross(&self, it: &Self) -> Self {
		Self {
			x: (self.y * it.z) - (self.z * it.y),
			y: (self.z * it.x) - (self.x * it.z),
			z: (self.x * it.y) - (self.y * it.x),
		}
	}

	pub fn normalize(&self) -> Self {
		let len = self.length();
		if len > T::ZERO {
			Self {
				x: self.x / len,
				y: self.y / len,
				z: self.z / len,
			}
		} else {
			Self::ZERO
		}
	}

	pub fn lerp(&self, it: &Self, t: T) -> Self {
		Self {
			x: self.x + (it.x - self.x) * t,
			y: self.y + (it.y - self.y) * t,
			z: self.z + (it.z - self.z) * t,
		}
	}


	pub fn clamp(&self, min: &Self, max: &Self) -> Self {
		Self {
			x: T::clamp(self.x, min.x, max.x),
			y: T::clamp(self.y, min.y, max.y),
			z: T::clamp(self.z, min.z, max.z),
		}
	}

	pub fn min(&self, it: &Self) -> Self {
		Self {
			x: T::min(self.x, it.x),
			y: T::min(self.y, it.y),
			z: T::min(self.z, it.z),
		}
	}

	pub fn max(&self, it: &Self) -> Self {
		Self {
			x: T::max(self.x, it.x),
			y: T::max(self.y, it.y),
			z: T::max(self.z, it.z),
		}
	}


	pub fn xy(&self) -> Vec2<T> {
		Vec2::new(self.x, self.y)
	}

	pub fn xz(&self) -> Vec2<T> {
		Vec2::new(self.x, self.z)
	}

	pub fn yz(&self) -> Vec2<T> {
		Vec2::new(self.y, self.z)
	}


	pub fn to_array(&self) -> [T; 3] {
		[self.x, self.y, self.z]
	}

	pub fn to_tuple(&self) -> (T, T, T) {
		(self.x, self.y, self.z)
	}
}


impl<T: Number> Default for Vec3<T> {
	fn default() -> Self {
		Self::ZERO
	}
}


impl<T: Number> From<[T; 3]> for Vec3<T> {
	fn from(arr: [T; 3]) -> Self {
		Self::new(arr[0], arr[1], arr[2])
	}
}

impl<T: Number> From<(T, T, T)> for Vec3<T> {
	fn from(tup: (T, T, T)) -> Self {
		Self::new(tup.0, tup.1, tup.2)
	}
}

impl<T: Number> From<Vec3<T>> for [T; 3] {
	fn from(v: Vec3<T>) -> Self {
		[v.x, v.y, v.z]
	}
}


impl<T: Number> Index<usize> for Vec3<T> {
	type Output = T;
	fn index(&self, index: usize) -> &Self::Output {
		match index {
			0 => &self.x,
			1 => &self.y,
			2 => &self.z,
			_ => panic!("Index out of bounds for Vec3"),
		}
	}
}

impl<T: Number> IndexMut<usize> for Vec3<T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		match index {
			0 => &mut self.x,
			1 => &mut self.y,
			2 => &mut self.z,
			_ => panic!("Index out of bounds for Vec3"),
		}
	}
}


impl<T: Number + Neg<Output = T>> Neg for Vec3<T> {
	type Output = Self;
	fn neg(self) -> Self::Output {
		Self {
			x: -self.x,
			y: -self.y,
			z: -self.z,
		}
	}
}


impl<T: Number> Add<Self> for Vec3<T> {
	type Output = Self;
	fn add(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z,
		}
	}
}

impl<T: Number> Sub<Self> for Vec3<T> {
	type Output = Self;
	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
			z: self.z - rhs.z,
		}
	}
}


impl<T: Number> Mul<Self> for Vec3<T> {
	type Output = Self;
	fn mul(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x * rhs.x,
			y: self.y * rhs.y,
			z: self.z * rhs.z,
		}
	}
}

impl<T: Number> Div<Self> for Vec3<T> {
	type Output = Self;
	fn div(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x / rhs.x,
			y: self.y / rhs.y,
			z: self.z / rhs.z,
		}
	}
}


impl<T: Number> Mul<T> for Vec3<T> {
	type Output = Self;
	fn mul(self, rhs: T) -> Self::Output {
		Self {
			x: self.x * rhs,
			y: self.y * rhs,
			z: self.z * rhs,
		}
	}
}

impl<T: Number> Div<T> for Vec3<T> {
	type Output = Self;
	fn div(self, rhs: T) -> Self::Output {
		Self {
			x: self.x / rhs,
			y: self.y / rhs,
			z: self.z / rhs,
		}
	}
}


impl<T: Number> AddAssign for Vec3<T> {
	fn add_assign(&mut self, rhs: Self) {
		self.x = self.x + rhs.x;
		self.y = self.y + rhs.y;
		self.z = self.z + rhs.z;
	}
}

impl<T: Number> SubAssign for Vec3<T> {
	fn sub_assign(&mut self, rhs: Self) {
		self.x = self.x - rhs.x;
		self.y = self.y - rhs.y;
		self.z = self.z - rhs.z;
	}
}


impl<T: Number> MulAssign<T> for Vec3<T> {
	fn mul_assign(&mut self, rhs: T) {
		self.x = self.x * rhs;
		self.y = self.y * rhs;
		self.z = self.z * rhs;
	}
}

impl<T: Number> DivAssign<T> for Vec3<T> {
	fn div_assign(&mut self, rhs: T) {
		self.x = self.x / rhs;
		self.y = self.y / rhs;
		self.z = self.z / rhs;
	}
}
