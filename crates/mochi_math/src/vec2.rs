/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use core::ops::{Add, Div, Mul, Neg, Sub};

use num_traits::Float;


pub struct Vec2<T>
where
	T: Float,
{
	pub x: T,
	pub y: T,
}

impl<T> Vec2<T>
where
	T: Default + Float,
{
	pub fn new() -> Self {
		Self {
			x: T::default(),
			y: T::default(),
		}
	}

	pub fn new_with(x: T, y: T) -> Self {
		Self { x: x, y: y }
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
		Self {
			x: self.x / len,
			y: self.y / len,
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
}


impl<T> Neg for Vec2<T>
where
	T: Float,
{
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self {
			x: self.x.neg(),
			y: self.y.neg(),
		}
	}
}


impl<T> Add<Self> for Vec2<T>
where
	T: Float,
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
	T: Float,
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
	T: Float,
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
	T: Float,
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
	T: Float,
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
	T: Float,
{
	type Output = Vec2<T>;

	fn div(self, rhs: T) -> Self::Output {
		Self {
			x: self.x / rhs,
			y: self.y / rhs,
		}
	}
}
