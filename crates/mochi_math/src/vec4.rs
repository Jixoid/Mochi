/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use core::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign};
use crate::{Number, Vec2, Vec3};


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec4<T>
where
	T: Number,
{
	pub x: T,
	pub y: T,
	pub z: T,
	pub w: T,
}

impl<T> Vec4<T>
where
	T: Number,
{
	pub const ZERO: Self = Self{x: T::ZERO, y: T::ZERO, z: T::ZERO, w: T::ZERO};
	pub const ONE: Self = Self{x: T::ONE, y: T::ONE, z: T::ONE, w: T::ONE};


	pub fn new_with(x: T, y: T, z: T, w: T) -> Self {
		Self { x, y, z, w }
	}

	pub fn new_cast(v: T) -> Self {
		Self { x: v, y: v, z: v, w: v }
	}


	pub fn zero() -> Self {
		Self::new_cast(T::zero())
	}

	pub fn one() -> Self {
		Self::new_cast(T::one())
	}


	pub fn dot(&self, it: &Self) -> T {
		(self.x * it.x) + (self.y * it.y) + (self.z * it.z) + (self.w * it.w)
	}

	pub fn dist_sq(&self, it: &Self) -> T {
		let dx = self.x - it.x;
		let dy = self.y - it.y;
		let dz = self.z - it.z;
		let dw = self.w - it.w;
		(dx * dx) + (dy * dy) + (dz * dz) + (dw * dw)
	}

	pub fn dist(&self, it: &Self) -> T {
		self.dist_sq(it).sqrt()
	}

	pub fn length(&self) -> T {
		self.dot(self).sqrt()
	}

	pub fn normalize(&self) -> Self {
		let len = self.length();
		if len > T::zero() {
			Self {
				x: self.x / len,
				y: self.y / len,
				z: self.z / len,
				w: self.w / len,
			}
		} else {
			Self::zero()
		}
	}

	pub fn lerp(&self, it: &Self, t: T) -> Self {
		Self {
			x: self.x + (it.x - self.x) * t,
			y: self.y + (it.y - self.y) * t,
			z: self.z + (it.z - self.z) * t,
			w: self.w + (it.w - self.w) * t,
		}
	}


	pub fn clamp(&self, min: &Self, max: &Self) -> Self {
		Self {
			x: T::clamp(self.x, min.x, max.x),
			y: T::clamp(self.y, min.y, max.y),
			z: T::clamp(self.z, min.z, max.z),
			w: T::clamp(self.w, min.w, max.w),
		}
	}

	pub fn min(&self, it: &Self) -> Self {
		Self {
			x: T::min(self.x, it.x),
			y: T::min(self.y, it.y),
			z: T::min(self.z, it.z),
			w: T::min(self.w, it.w),
		}
	}

	pub fn max(&self, it: &Self) -> Self {
		Self {
			x: T::max(self.x, it.x),
			y: T::max(self.y, it.y),
			z: T::max(self.z, it.z),
			w: T::max(self.w, it.w),
		}
	}


	pub fn xyz(&self) -> Vec3<T> {
		Vec3::new_with(self.x, self.y, self.z)
	}

	pub fn xy(&self) -> Vec2<T> {
		Vec2::new_with(self.x, self.y)
	}


	pub fn to_array(&self) -> [T; 4] {
		[self.x, self.y, self.z, self.w]
	}
	
	pub fn to_tuple(&self) -> (T, T, T, T) {
		(self.x, self.y, self.z, self.w)
	}
}


impl<T> Default for Vec4<T>
where
	T: Number,
{
	fn default() -> Self {
		Self::ZERO
	}
}


impl<T> From<[T; 4]> for Vec4<T>
where
	T: Number,
{
	fn from(arr: [T; 4]) -> Self {
		Self::new_with(arr[0], arr[1], arr[2], arr[3])
	}
}

impl<T> From<(T, T, T, T)> for Vec4<T>
where
	T: Number,
{
	fn from(tup: (T, T, T, T)) -> Self {
		Self::new_with(tup.0, tup.1, tup.2, tup.3)
	}
}

impl<T> From<Vec4<T>> for [T; 4]
where
	T: Number,
{
	fn from(v: Vec4<T>) -> Self {
		[v.x, v.y, v.z, v.w]
	}
}


impl<T> Index<usize> for Vec4<T>
where
	T: Number,
{
	type Output = T;
	fn index(&self, index: usize) -> &Self::Output {
		match index {
			0 => &self.x,
			1 => &self.y,
			2 => &self.z,
			3 => &self.w,
			_ => panic!("Index out of bounds for Vec4"),
		}
	}
}

impl<T> IndexMut<usize> for Vec4<T>
where
	T: Number,
{
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		match index {
			0 => &mut self.x,
			1 => &mut self.y,
			2 => &mut self.z,
			3 => &mut self.w,
			_ => panic!("Index out of bounds for Vec4"),
		}
	}
}


impl<T> Neg for Vec4<T>
where
	T: Number + Neg<Output = T>,
{
	type Output = Self;
	fn neg(self) -> Self::Output {
		Self {
			x: -self.x,
			y: -self.y,
			z: -self.z,
			w: -self.w,
		}
	}
}


impl<T> Add<Self> for Vec4<T>
where
	T: Number,
{
	type Output = Self;
	fn add(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z,
			w: self.w + rhs.w,
		}
	}
}

impl<T> Sub<Self> for Vec4<T>
where
	T: Number,
{
	type Output = Self;
	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
			z: self.z - rhs.z,
			w: self.w - rhs.w,
		}
	}
}


impl<T> Mul<Self> for Vec4<T>
where
	T: Number,
{
	type Output = Self;
	fn mul(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x * rhs.x,
			y: self.y * rhs.y,
			z: self.z * rhs.z,
			w: self.w * rhs.w,
		}
	}
}

impl<T> Div<Self> for Vec4<T>
where
	T: Number,
{
	type Output = Self;
	fn div(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x / rhs.x,
			y: self.y / rhs.y,
			z: self.z / rhs.z,
			w: self.w / rhs.w,
		}
	}
}


impl<T> Mul<T> for Vec4<T>
where
	T: Number,
{
	type Output = Self;
	fn mul(self, rhs: T) -> Self::Output {
		Self {
			x: self.x * rhs,
			y: self.y * rhs,
			z: self.z * rhs,
			w: self.w * rhs,
		}
	}
}

impl<T> Div<T> for Vec4<T>
where
	T: Number,
{
	type Output = Self;
	fn div(self, rhs: T) -> Self::Output {
		Self {
			x: self.x / rhs,
			y: self.y / rhs,
			z: self.z / rhs,
			w: self.w / rhs,
		}
	}
}


impl<T> AddAssign for Vec4<T>
where
	T: Number,
{
	fn add_assign(&mut self, rhs: Self) {
		self.x = self.x + rhs.x;
		self.y = self.y + rhs.y;
		self.z = self.z + rhs.z;
		self.w = self.w + rhs.w;
	}
}

impl<T> SubAssign for Vec4<T>
where
	T: Number,
{
	fn sub_assign(&mut self, rhs: Self) {
		self.x = self.x - rhs.x;
		self.y = self.y - rhs.y;
		self.z = self.z - rhs.z;
		self.w = self.w - rhs.w;
	}
}


impl<T> MulAssign<T> for Vec4<T>
where
	T: Number,
{
	fn mul_assign(&mut self, rhs: T) {
		self.x = self.x * rhs;
		self.y = self.y * rhs;
		self.z = self.z * rhs;
		self.w = self.w * rhs;
	}
}

impl<T> DivAssign<T> for Vec4<T>
where
	T: Number,
{
	fn div_assign(&mut self, rhs: T) {
		self.x = self.x / rhs;
		self.y = self.y / rhs;
		self.z = self.z / rhs;
		self.w = self.w / rhs;
	}
}
