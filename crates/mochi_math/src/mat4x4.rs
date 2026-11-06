/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use core::ops::{Index, IndexMut, Mul};
use crate::{Float, Mat3x4, Number, Rad, Vec3, Vec4};


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mat4x4<T: Number = f32> {
	pub rows: [Vec4<T>; 4],
}

pub type Mat4<T = f32> = Mat4x4<T>;


impl<T: Number> Mat4x4<T> {
	pub const ZERO: Self = Self {
		rows: [Vec4::ZERO, Vec4::ZERO, Vec4::ZERO, Vec4::ZERO],
	};

	pub const IDENTITY: Self = Self {
		rows: [
			Vec4::new_with(T::ONE, T::ZERO, T::ZERO, T::ZERO),
			Vec4::new_with(T::ZERO, T::ONE, T::ZERO, T::ZERO),
			Vec4::new_with(T::ZERO, T::ZERO, T::ONE, T::ZERO),
			Vec4::new_with(T::ZERO, T::ZERO, T::ZERO, T::ONE),
		],
	};

	pub const fn new(r0: Vec4<T>, r1: Vec4<T>, r2: Vec4<T>, r3: Vec4<T>) -> Self {
		Self {
			rows: [r0, r1, r2, r3],
		}
	}

	pub const fn new_with(r0: Vec4<T>, r1: Vec4<T>, r2: Vec4<T>, r3: Vec4<T>) -> Self {
		Self::new(r0, r1, r2, r3)
	}

	pub const fn from_cols(c0: Vec4<T>, c1: Vec4<T>, c2: Vec4<T>, c3: Vec4<T>) -> Self {
		Self {
			rows: [
				Vec4::new_with(c0.x, c1.x, c2.x, c3.x),
				Vec4::new_with(c0.y, c1.y, c2.y, c3.y),
				Vec4::new_with(c0.z, c1.z, c2.z, c3.z),
				Vec4::new_with(c0.w, c1.w, c2.w, c3.w),
			],
		}
	}

	pub fn to_array(&self) -> [T; 16] {
		[
			self.rows[0].x, self.rows[0].y, self.rows[0].z, self.rows[0].w,
			self.rows[1].x, self.rows[1].y, self.rows[1].z, self.rows[1].w,
			self.rows[2].x, self.rows[2].y, self.rows[2].z, self.rows[2].w,
			self.rows[3].x, self.rows[3].y, self.rows[3].z, self.rows[3].w,
		]
	}

	pub fn transpose(&self) -> Self {
		Self::from_cols(self.rows[0], self.rows[1], self.rows[2], self.rows[3])
	}
}

impl<T: Float> Mat4x4<T> {
	pub fn perspective(fov_deg: T, aspect: T, near_z: T, far_z: T) -> Self {
		let rad = fov_deg.to_radians();
		Self::perspective_rad(Rad::new(rad), aspect, near_z, far_z)
	}

	pub fn perspective_rad(fov: Rad<T>, aspect: T, near_z: T, far_z: T) -> Self {
		let mut res = Self::ZERO;
		let tan_half_fov = (fov * T::HALF).tan();

		// Vulkan standard depth clip [0, 1] with inverted Y for Vulkan NDC
		res.rows[0].x = T::ONE / (aspect * tan_half_fov);
		res.rows[1].y = -(T::ONE / tan_half_fov);
		res.rows[2].z = -far_z / (far_z - near_z);
		res.rows[2].w = -(far_z * near_z) / (far_z - near_z);
		res.rows[3].z = -T::ONE;
		res.rows[3].w = T::ZERO;

		res
	}

	pub fn orthographic(left: T, right: T, bottom: T, top: T, near_z: T, far_z: T) -> Self {
		let mut res = Self::ZERO;
		let two = T::TWO;

		res.rows[0].x = two / (right - left);
		res.rows[0].w = -(right + left) / (right - left);

		res.rows[1].y = -two / (top - bottom); // Inverted Y for Vulkan
		res.rows[1].w = -(top + bottom) / (top - bottom);

		res.rows[2].z = -T::ONE / (far_z - near_z);
		res.rows[2].w = -near_z / (far_z - near_z);

		res.rows[3].w = T::ONE;

		res
	}

	pub fn look_at(eye: Vec3<T>, center: Vec3<T>, up: Vec3<T>) -> Self {
		let f = (center - eye).normalize();

		let s = Vec3::new_with(
			f.y * up.z - f.z * up.y,
			f.z * up.x - f.x * up.z,
			f.x * up.y - f.y * up.x,
		)
		.normalize();

		let u = Vec3::new_with(
			s.y * f.z - s.z * f.y,
			s.z * f.x - s.x * f.z,
			s.x * f.y - s.y * f.x,
		);

		let mut res = Self::ZERO;
		res.rows[0] = Vec4::new_with(s.x, s.y, s.z, -(s.x * eye.x + s.y * eye.y + s.z * eye.z));
		res.rows[1] = Vec4::new_with(u.x, u.y, u.z, -(u.x * eye.x + u.y * eye.y + u.z * eye.z));
		res.rows[2] = Vec4::new_with(-f.x, -f.y, -f.z, f.x * eye.x + f.y * eye.y + f.z * eye.z);
		res.rows[3] = Vec4::new_with(T::ZERO, T::ZERO, T::ZERO, T::ONE);

		res
	}

	pub fn inverse(&self) -> Self {
		let mut a = [
			[self.rows[0].x, self.rows[0].y, self.rows[0].z, self.rows[0].w],
			[self.rows[1].x, self.rows[1].y, self.rows[1].z, self.rows[1].w],
			[self.rows[2].x, self.rows[2].y, self.rows[2].z, self.rows[2].w],
			[self.rows[3].x, self.rows[3].y, self.rows[3].z, self.rows[3].w],
		];
		let mut b = [
			[T::ONE, T::ZERO, T::ZERO, T::ZERO],
			[T::ZERO, T::ONE, T::ZERO, T::ZERO],
			[T::ZERO, T::ZERO, T::ONE, T::ZERO],
			[T::ZERO, T::ZERO, T::ZERO, T::ONE],
		];

		let eps = T::from_f32(1e-12);

		for k in 0..4 {
			let mut pivot = k;
			let mut max_val = a[k][k].abs();
			for i in (k + 1)..4 {
				let val = a[i][k].abs();
				if val > max_val {
					max_val = val;
					pivot = i;
				}
			}

			if max_val < eps {
				return Self::ZERO;
			}

			if pivot != k {
				a.swap(k, pivot);
				b.swap(k, pivot);
			}

			let pv = a[k][k];
			let inv_pv = T::ONE / pv;

			for j in 0..4 {
				a[k][j] = a[k][j] * inv_pv;
				b[k][j] = b[k][j] * inv_pv;
			}

			for i in 0..4 {
				if i != k {
					let factor = a[i][k];
					if factor.abs() > eps {
						for j in 0..4 {
							a[i][j] = a[i][j] - factor * a[k][j];
							b[i][j] = b[i][j] - factor * b[k][j];
						}
					}
				}
			}
		}

		Self::new(
			Vec4::new_with(b[0][0], b[0][1], b[0][2], b[0][3]),
			Vec4::new_with(b[1][0], b[1][1], b[1][2], b[1][3]),
			Vec4::new_with(b[2][0], b[2][1], b[2][2], b[2][3]),
			Vec4::new_with(b[3][0], b[3][1], b[3][2], b[3][3]),
		)
	}
}


impl<T: Number> Default for Mat4x4<T> {
	fn default() -> Self {
		Self::IDENTITY
	}
}


impl<T: Number> Index<usize> for Mat4x4<T> {
	type Output = Vec4<T>;
	fn index(&self, index: usize) -> &Self::Output {
		&self.rows[index]
	}
}

impl<T: Number> IndexMut<usize> for Mat4x4<T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.rows[index]
	}
}


impl<T: Number> Mul<Self> for Mat4x4<T> {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		let mut res = Self::ZERO;
		for i in 0..4 {
			let x = self.rows[i].x;
			let y = self.rows[i].y;
			let z = self.rows[i].z;
			let w = self.rows[i].w;

			res.rows[i] = Vec4::new_with(
				x * rhs.rows[0].x + y * rhs.rows[1].x + z * rhs.rows[2].x + w * rhs.rows[3].x,
				x * rhs.rows[0].y + y * rhs.rows[1].y + z * rhs.rows[2].y + w * rhs.rows[3].y,
				x * rhs.rows[0].z + y * rhs.rows[1].z + z * rhs.rows[2].z + w * rhs.rows[3].z,
				x * rhs.rows[0].w + y * rhs.rows[1].w + z * rhs.rows[2].w + w * rhs.rows[3].w,
			);
		}
		res
	}
}

impl<T: Number> Mul<Vec4<T>> for Mat4x4<T> {
	type Output = Vec4<T>;

	fn mul(self, v: Vec4<T>) -> Self::Output {
		Vec4::new_with(
			self.rows[0].x * v.x + self.rows[0].y * v.y + self.rows[0].z * v.z + self.rows[0].w * v.w,
			self.rows[1].x * v.x + self.rows[1].y * v.y + self.rows[1].z * v.z + self.rows[1].w * v.w,
			self.rows[2].x * v.x + self.rows[2].y * v.y + self.rows[2].z * v.z + self.rows[2].w * v.w,
			self.rows[3].x * v.x + self.rows[3].y * v.y + self.rows[3].z * v.z + self.rows[3].w * v.w,
		)
	}
}

impl<T: Number> Mul<Vec3<T>> for Mat4x4<T> {
	type Output = Vec3<T>;

	fn mul(self, v: Vec3<T>) -> Self::Output {
		let v4: Vec4<T> = self * Vec4::new(v.x, v.y, v.z, T::ONE);
		Vec3::new(v4.x, v4.y, v4.z)
	}
}


impl<T: Number> From<[Vec4<T>; 4]> for Mat4x4<T> {
	fn from(rows: [Vec4<T>; 4]) -> Self {
		Self { rows }
	}
}

impl<T: Number> From<[T; 16]> for Mat4x4<T> {
	fn from(arr: [T; 16]) -> Self {
		Self {
			rows: [
				Vec4::new(arr[0], arr[1], arr[2], arr[3]),
				Vec4::new(arr[4], arr[5], arr[6], arr[7]),
				Vec4::new(arr[8], arr[9], arr[10], arr[11]),
				Vec4::new(arr[12], arr[13], arr[14], arr[15]),
			],
		}
	}
}

impl<T: Number> From<Mat4x4<T>> for [T; 16] {
	fn from(m: Mat4x4<T>) -> Self {
		m.to_array()
	}
}

impl<T: Number> From<Mat3x4<T>> for Mat4x4<T> {
	fn from(m: Mat3x4<T>) -> Self {
		m.to_mat4()
	}
}

