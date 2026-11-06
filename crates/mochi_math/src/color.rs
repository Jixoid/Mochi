/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use core::ops::{Add, Mul, Sub};
use crate::{Float, Number, Vec3, Vec4};


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color<T: Number = f32> {
	pub r: T,
	pub g: T,
	pub b: T,
	pub a: T,
}


impl<T: Number> Color<T> {
	pub const TRANSPARENT: Self = Self {r: T::ZERO, g: T::ZERO, b: T::ZERO, a: T::ZERO};

	pub const BLACK: Self = Self {r: T::ZERO, g: T::ZERO, b: T::ZERO, a: T::ONE};
	pub const WHITE: Self = Self {r: T::ONE, g: T::ONE, b: T::ONE, a: T::ONE};

	pub const RED: Self = Self {r: T::ONE, g: T::ZERO, b: T::ZERO, a: T::ONE};
	pub const GREEN: Self = Self {r: T::ZERO, g: T::ONE, b: T::ZERO, a: T::ONE};
	pub const BLUE: Self = Self {r: T::ZERO, g: T::ZERO, b: T::ONE, a: T::ONE};

	pub const YELLOW: Self = Self {r: T::ONE, g: T::ONE, b: T::ZERO, a: T::ONE};
	pub const CYAN: Self = Self {r: T::ZERO, g: T::ONE, b: T::ONE, a: T::ONE};
	pub const MAGENTA: Self = Self {r: T::ONE, g: T::ZERO, b: T::ONE, a: T::ONE};


	pub const fn new(r: T, g: T, b: T, a: T) -> Self {
		Self { r, g, b, a }
	}

	pub fn rgb(r: T, g: T, b: T) -> Self {
		Self { r, g, b, a: T::ONE }
	}


	pub fn lerp(&self, other: &Self, t: T) -> Self {
		Self {
			r: self.r + (other.r - self.r) * t,
			g: self.g + (other.g - self.g) * t,
			b: self.b + (other.b - self.b) * t,
			a: self.a + (other.a - self.a) * t,
		}
	}


	pub fn to_array(&self) -> [T; 4] {
		[self.r, self.g, self.b, self.a]
	}

	pub fn to_tuple(&self) -> (T, T, T, T) {
		(self.r, self.g, self.b, self.a)
	}
}

impl<T: Float> Color<T> {
	pub fn from_hex(hex: u32) -> Self {
		let r = ((hex >> 24) & 0xFF) as f32 / 255.0;
		let g = ((hex >> 16) & 0xFF) as f32 / 255.0;
		let b = ((hex >> 8) & 0xFF) as f32 / 255.0;
		let a = (hex & 0xFF) as f32 / 255.0;

		Self {
			r: T::from_f32(r),
			g: T::from_f32(g),
			b: T::from_f32(b),
			a: T::from_f32(a),
		}
	}

	pub fn to_rgba_u8(&self) -> [u8; 4] {
		let clamp_u8 = |v: T| -> u8 {
			let clamped = T::clamp(v, T::ZERO, T::ONE);
			(clamped * T::from_f32(255.0)).to_f32() as u8
		};

		[
			clamp_u8(self.r),
			clamp_u8(self.g),
			clamp_u8(self.b),
			clamp_u8(self.a),
		]
	}
}


impl<T: Number> Default for Color<T> {
	fn default() -> Self {
		Self::WHITE
	}
}


impl<T: Number> Add<Self> for Color<T> {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			r: self.r + rhs.r,
			g: self.g + rhs.g,
			b: self.b + rhs.b,
			a: self.a + rhs.a,
		}
	}
}

impl<T: Number> Sub<Self> for Color<T> {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			r: self.r - rhs.r,
			g: self.g - rhs.g,
			b: self.b - rhs.b,
			a: self.a - rhs.a,
		}
	}
}


impl<T: Number> Mul<Self> for Color<T> {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		Self {
			r: self.r * rhs.r,
			g: self.g * rhs.g,
			b: self.b * rhs.b,
			a: self.a * rhs.a,
		}
	}
}

impl<T: Number> Mul<T> for Color<T> {
	type Output = Self;

	fn mul(self, rhs: T) -> Self::Output {
		Self {
			r: self.r * rhs,
			g: self.g * rhs,
			b: self.b * rhs,
			a: self.a * rhs,
		}
	}
}


impl<T: Number> From<Vec4<T>> for Color<T> {
	fn from(v: Vec4<T>) -> Self {
		Self {
			r: v.x,
			g: v.y,
			b: v.z,
			a: v.w,
		}
	}
}

impl<T: Number> From<Color<T>> for Vec4<T> {
	fn from(c: Color<T>) -> Self {
		Vec4::new(c.r, c.g, c.b, c.a)
	}
}


impl<T: Number> From<Vec3<T>> for Color<T> {
	fn from(v: Vec3<T>) -> Self {
		Self {
			r: v.x,
			g: v.y,
			b: v.z,
			a: T::ONE,
		}
	}
}

impl<T: Number> From<Color<T>> for Vec3<T> {
	fn from(c: Color<T>) -> Self {
		Vec3::new(c.r, c.g, c.b)
	}
}

impl<T: Number> From<[T; 4]> for Color<T> {
	fn from(arr: [T; 4]) -> Self {
		Self::new(arr[0], arr[1], arr[2], arr[3])
	}
}

impl<T: Number> From<[T; 3]> for Color<T> {
	fn from(arr: [T; 3]) -> Self {
		Self::rgb(arr[0], arr[1], arr[2])
	}
}

impl<T: Number> From<(T, T, T, T)> for Color<T> {
	fn from((r, g, b, a): (T, T, T, T)) -> Self {
		Self::new(r, g, b, a)
	}
}

impl<T: Number> From<(T, T, T)> for Color<T> {
	fn from((r, g, b): (T, T, T)) -> Self {
		Self::rgb(r, g, b)
	}
}

impl<T: Number> From<Color<T>> for [T; 4] {
	fn from(c: Color<T>) -> Self {
		c.to_array()
	}
}

impl<T: Number> From<Color<T>> for (T, T, T, T) {
	fn from(c: Color<T>) -> Self {
		(c.r, c.g, c.b, c.a)
	}
}

impl From<u32> for Color<f32> {
	fn from(hex: u32) -> Self {
		Self::from_hex(hex)
	}
}
