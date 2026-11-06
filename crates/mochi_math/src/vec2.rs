/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use core::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign};
use crate::Number;


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec2<T>
where
	T: Number,
{
	pub x: T,
	pub y: T,
}

impl<T> Vec2<T>
where
	T: Number,
{
	pub const ZERO: Self = Self{x: T::ZERO, y: T::ZERO};
	pub const ONE: Self = Self{x: T::ONE, y: T::ONE};

	pub fn new_with(x: T, y: T) -> Self {
		Self { x, y }
	}

	pub fn new_cast(v: T) -> Self {
		Self { x: v, y: v }
	}


	pub fn dot(&self, it: &Self) -> T {
		(self.x * it.x) + (self.y * it.y)
	}

	pub fn dist_sq(&self, it: &Self) -> T {
		let dx = self.x - it.x;
		let dy = self.y - it.y;
		(dx * dx) + (dy * dy)
	}

	pub fn dist(&self, it: &Self) -> T {
		self.dist_sq(it).sqrt()
	}

	pub fn length(&self) -> T {
		self.dot(self).sqrt()
	}

	pub fn cross(&self, it: &Self) -> T {
		(self.x * it.y) - (self.y * it.x)
	}

	pub fn normalize(&self) -> Self {
		let len = self.length();
		if len > T::ZERO {
			Self {
				x: self.x / len,
				y: self.y / len,
			}
		} else {
			Self::ZERO
		}
	}

	pub fn lerp(&self, it: &Self, t: T) -> Self {
		Self {
			x: self.x + (it.x - self.x) * t,
			y: self.y + (it.y - self.y) * t,
		}
	}

	
	pub fn clamp(&self, min: &Self, max: &Self) -> Self {
		Self {
			x: T::clamp(self.x, min.x, max.x),
			y: T::clamp(self.y, min.y, max.y),
		}
	}

	pub fn min(&self, it: &Self) -> Self {
		Self {
			x: T::min(self.x, it.x),
			y: T::min(self.y, it.y),
		}
	}

	pub fn max(&self, it: &Self) -> Self {
		Self {
			x: T::max(self.x, it.x),
			y: T::max(self.y, it.y),
		}
	}


	pub fn to_array(&self) -> [T; 2] {
		[self.x, self.y]
	}
	
	pub fn to_tuple(&self) -> (T, T) {
		(self.x, self.y)
	}
}


impl<T> Default for Vec2<T>
where
	T: Number,
{
	fn default() -> Self {
		Self::ZERO
	}
}


impl<T> From<[T; 2]> for Vec2<T>
where
	T: Number,
{
	fn from(arr: [T; 2]) -> Self {
		Self::new_with(arr[0], arr[1])
	}
}

impl<T> From<(T, T)> for Vec2<T>
where
	T: Number,
{
	fn from(tup: (T, T)) -> Self {
		Self::new_with(tup.0, tup.1)
	}
}

impl<T> From<Vec2<T>> for [T; 2]
where
	T: Number,
{
	fn from(v: Vec2<T>) -> Self {
		[v.x, v.y]
	}
}


impl<T> Index<usize> for Vec2<T>
where
	T: Number,
{
	type Output = T;
	fn index(&self, index: usize) -> &Self::Output {
		match index {
			0 => &self.x,
			1 => &self.y,
			_ => panic!("Index out of bounds for Vec2"),
		}
	}
}

impl<T> IndexMut<usize> for Vec2<T>
where
	T: Number,
{
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		match index {
			0 => &mut self.x,
			1 => &mut self.y,
			_ => panic!("Index out of bounds for Vec2"),
		}
	}
}


impl<T> Neg for Vec2<T>
where
	T: Number + Neg<Output = T>,
{
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self {
			x: -self.x,
			y: -self.y,
		}
	}
}


impl<T> Add<Self> for Vec2<T>
where
	T: Number,
{
	type Output = Vec2<T>;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

impl<T> Sub<Self> for Vec2<T>
where
	T: Number,
{
	type Output = Vec2<T>;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
		}
	}
}


impl<T> Mul<Self> for Vec2<T>
where
	T: Number,
{
	type Output = Vec2<T>;

	fn mul(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x * rhs.x,
			y: self.y * rhs.y,
		}
	}
}

impl<T> Div<Self> for Vec2<T>
where
	T: Number,
{
	type Output = Vec2<T>;

	fn div(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x / rhs.x,
			y: self.y / rhs.y,
		}
	}
}


impl<T> Mul<T> for Vec2<T>
where
	T: Number,
{
	type Output = Vec2<T>;

	fn mul(self, rhs: T) -> Self::Output {
		Self {
			x: self.x * rhs,
			y: self.y * rhs,
		}
	}
}

impl<T> Div<T> for Vec2<T>
where
	T: Number,
{
	type Output = Vec2<T>;

	fn div(self, rhs: T) -> Self::Output {
		Self {
			x: self.x / rhs,
			y: self.y / rhs,
		}
	}
}


impl<T> AddAssign for Vec2<T>
where
	T: Number,
{
	fn add_assign(&mut self, rhs: Self) {
		self.x = self.x + rhs.x;
		self.y = self.y + rhs.y;
	}
}

impl<T> SubAssign for Vec2<T>
where
	T: Number,
{
	fn sub_assign(&mut self, rhs: Self) {
		self.x = self.x - rhs.x;
		self.y = self.y - rhs.y;
	}
}


impl<T> MulAssign<T> for Vec2<T>
where
	T: Number,
{
	fn mul_assign(&mut self, rhs: T) {
		self.x = self.x * rhs;
		self.y = self.y * rhs;
	}
}

impl<T> DivAssign<T> for Vec2<T>
where
	T: Number,
{
	fn div_assign(&mut self, rhs: T) {
		self.x = self.x / rhs;
		self.y = self.y / rhs;
	}
}
