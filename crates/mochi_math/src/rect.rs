/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{Number, Vec2};


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rect<T: Number = f32> {
	pub min: Vec2<T>,
	pub max: Vec2<T>,
}


impl<T: Number> Rect<T> {
	pub const ZERO: Self = Self {
		min: Vec2::ZERO,
		max: Vec2::ZERO,
	};


	pub fn new(x: T, y: T, width: T, height: T) -> Self {
		Self {
			min: Vec2::new(x, y),
			max: Vec2::new(x + width, y + height),
		}
	}

	pub fn from_min_max(min: Vec2<T>, max: Vec2<T>) -> Self {
		Self { min, max }
	}

	pub fn from_center_size(center: Vec2<T>, size: Vec2<T>) -> Self {
		let half = T::ONE / (T::ONE + T::ONE);
		let half_size = size * half;
		Self {
			min: center - half_size,
			max: center + half_size,
		}
	}


	pub fn x(&self) -> T {
		self.min.x
	}

	pub fn y(&self) -> T {
		self.min.y
	}

	pub fn width(&self) -> T {
		self.max.x - self.min.x
	}

	pub fn height(&self) -> T {
		self.max.y - self.min.y
	}


	pub fn size(&self) -> Vec2<T> {
		self.max - self.min
	}

	pub fn center(&self) -> Vec2<T> {
		let half = T::ONE / (T::ONE + T::ONE);
		(self.min + self.max) * half
	}


	pub fn contains_point(&self, p: Vec2<T>) -> bool {
		p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
	}

	pub fn intersects(&self, other: &Self) -> bool {
		self.min.x <= other.max.x
			&& self.max.x >= other.min.x
			&& self.min.y <= other.max.y
			&& self.max.y >= other.min.y
	}

	pub fn intersection(&self, other: &Self) -> Option<Self> {
		if !self.intersects(other) {
			return None;
		}

		Some(Self {
			min: Vec2::new(
				T::max(self.min.x, other.min.x),
				T::max(self.min.y, other.min.y),
			),
			max: Vec2::new(
				T::min(self.max.x, other.max.x),
				T::min(self.max.y, other.max.y),
			),
		})
	}

	pub fn union(&self, other: &Self) -> Self {
		Self {
			min: Vec2::new(
				T::min(self.min.x, other.min.x),
				T::min(self.min.y, other.min.y),
			),
			max: Vec2::new(
				T::max(self.max.x, other.max.x),
				T::max(self.max.y, other.max.y),
			),
		}
	}
}


impl<T: Number> Default for Rect<T> {
	fn default() -> Self {
		Self::ZERO
	}
}


impl<T: Number> From<(Vec2<T>, Vec2<T>)> for Rect<T> {
	fn from((min, max): (Vec2<T>, Vec2<T>)) -> Self {
		Self::from_min_max(min, max)
	}
}

impl<T: Number> From<[Vec2<T>; 2]> for Rect<T> {
	fn from(arr: [Vec2<T>; 2]) -> Self {
		Self::from_min_max(arr[0], arr[1])
	}
}

impl<T: Number> From<[T; 4]> for Rect<T> {
	fn from(arr: [T; 4]) -> Self {
		Self::new(arr[0], arr[1], arr[2], arr[3])
	}
}

impl<T: Number> From<(T, T, T, T)> for Rect<T> {
	fn from((x, y, w, h): (T, T, T, T)) -> Self {
		Self::new(x, y, w, h)
	}
}
