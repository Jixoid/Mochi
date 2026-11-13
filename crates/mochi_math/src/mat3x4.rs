/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use core::ops::{Index, IndexMut, Mul};
use crate::{Float, Mat4x4, Number, Quat, Vec3, Vec4};


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mat3x4<T: Number = f32> {
	pub rows: [Vec4<T>; 3],
}

pub type Mat4x3<T = f32> = Mat3x4<T>;


impl<T: Number> Mat3x4<T> {
	pub const ZERO: Self = Self {
		rows: [Vec4::ZERO, Vec4::ZERO, Vec4::ZERO],
	};

	pub const IDENTITY: Self = Self {
		rows: [
			Vec4::new_with(T::ONE, T::ZERO, T::ZERO, T::ZERO),
			Vec4::new_with(T::ZERO, T::ONE, T::ZERO, T::ZERO),
			Vec4::new_with(T::ZERO, T::ZERO, T::ONE, T::ZERO),
		],
	};

	pub const fn new(r0: Vec4<T>, r1: Vec4<T>, r2: Vec4<T>) -> Self {
		Self { rows: [r0, r1, r2] }
	}

	pub const fn new_with(r0: Vec4<T>, r1: Vec4<T>, r2: Vec4<T>) -> Self {
		Self::new(r0, r1, r2)
	}

	pub const fn from_cols(c0: Vec3<T>, c1: Vec3<T>, c2: Vec3<T>, c3: Vec3<T>) -> Self {
		Self {
			rows: [
				Vec4::new_with(c0.x, c1.x, c2.x, c3.x),
				Vec4::new_with(c0.y, c1.y, c2.y, c3.y),
				Vec4::new_with(c0.z, c1.z, c2.z, c3.z),
			],
		}
	}

	pub fn to_array(&self) -> [T; 12] {
		[
			self.rows[0].x,
			self.rows[0].y,
			self.rows[0].z,
			self.rows[0].w,
			self.rows[1].x,
			self.rows[1].y,
			self.rows[1].z,
			self.rows[1].w,
			self.rows[2].x,
			self.rows[2].y,
			self.rows[2].z,
			self.rows[2].w,
		]
	}

	pub fn to_mat4(&self) -> Mat4x4<T> {
		Mat4x4::new(
			self.rows[0],
			self.rows[1],
			self.rows[2],
			Vec4::new_with(T::ZERO, T::ZERO, T::ZERO, T::ONE),
		)
	}
}

impl<T: Float> Mat3x4<T> {
	pub fn model(position: Vec3<T>, rotate: Quat<T>, scale: Vec3<T>) -> Self {
		let q = rotate.normalize();

		let xx = q.x * q.x;
		let yy = q.y * q.y;
		let zz = q.z * q.z;
		let xy = q.x * q.y;
		let xz = q.x * q.z;
		let yz = q.y * q.z;
		let wx = q.w * q.x;
		let wy = q.w * q.y;
		let wz = q.w * q.z;

		let one = T::ONE;
		let two = T::TWO;

		Self::new(
			Vec4::new_with(
				(one - two * (yy + zz)) * scale.x,
				(two * (xy - wz)) * scale.y,
				(two * (xz + wy)) * scale.z,
				position.x,
			),
			Vec4::new_with(
				(two * (xy + wz)) * scale.x,
				(one - two * (xx + zz)) * scale.y,
				(two * (yz - wx)) * scale.z,
				position.y,
			),
			Vec4::new_with(
				(two * (xz - wy)) * scale.x,
				(two * (yz + wx)) * scale.y,
				(one - two * (xx + yy)) * scale.z,
				position.z,
			),
		)
	}

	pub fn inverse(&self) -> Self {
		let c00 = self.rows[1].y * self.rows[2].z - self.rows[1].z * self.rows[2].y;
		let c01 = self.rows[1].z * self.rows[2].x - self.rows[1].x * self.rows[2].z;
		let c02 = self.rows[1].x * self.rows[2].y - self.rows[1].y * self.rows[2].x;

		let c10 = self.rows[0].z * self.rows[2].y - self.rows[0].y * self.rows[2].z;
		let c11 = self.rows[0].x * self.rows[2].z - self.rows[0].z * self.rows[2].x;
		let c12 = self.rows[0].y * self.rows[2].x - self.rows[0].x * self.rows[2].y;

		let c20 = self.rows[0].y * self.rows[1].z - self.rows[0].z * self.rows[1].y;
		let c21 = self.rows[0].z * self.rows[1].x - self.rows[0].x * self.rows[1].z;
		let c22 = self.rows[0].x * self.rows[1].y - self.rows[0].y * self.rows[1].x;

		let det = self.rows[0].x * c00 + self.rows[0].y * c01 + self.rows[0].z * c02;

		let eps = T::from_f32(1e-12);
		if det.abs() < eps {
			return Self::ZERO;
		}

		let inv_det = T::ONE / det;

		let r00 = c00 * inv_det;
		let r01 = c10 * inv_det;
		let r02 = c20 * inv_det;

		let r10 = c01 * inv_det;
		let r11 = c11 * inv_det;
		let r12 = c21 * inv_det;

		let r20 = c02 * inv_det;
		let r21 = c12 * inv_det;
		let r22 = c22 * inv_det;

		let tx = self.rows[0].w;
		let ty = self.rows[1].w;
		let tz = self.rows[2].w;

		let ntx = -(r00 * tx + r01 * ty + r02 * tz);
		let nty = -(r10 * tx + r11 * ty + r12 * tz);
		let ntz = -(r20 * tx + r21 * ty + r22 * tz);

		Self::new(
			Vec4::new_with(r00, r01, r02, ntx),
			Vec4::new_with(r10, r11, r12, nty),
			Vec4::new_with(r20, r21, r22, ntz),
		)
	}
}

impl<T: Number> Default for Mat3x4<T> {
	fn default() -> Self {
		Self::IDENTITY
	}
}

impl<T: Number> Index<usize> for Mat3x4<T> {
	type Output = Vec4<T>;
	fn index(&self, index: usize) -> &Self::Output {
		&self.rows[index]
	}
}

impl<T: Number> IndexMut<usize> for Mat3x4<T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.rows[index]
	}
}

impl<T: Number> Mul<Self> for Mat3x4<T> {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		let mut res = Self::ZERO;
		for i in 0..3 {
			let x = self.rows[i].x;
			let y = self.rows[i].y;
			let z = self.rows[i].z;
			let w = self.rows[i].w;

			res.rows[i] = Vec4::new_with(
				x * rhs.rows[0].x + y * rhs.rows[1].x + z * rhs.rows[2].x,
				x * rhs.rows[0].y + y * rhs.rows[1].y + z * rhs.rows[2].y,
				x * rhs.rows[0].z + y * rhs.rows[1].z + z * rhs.rows[2].z,
				x * rhs.rows[0].w + y * rhs.rows[1].w + z * rhs.rows[2].w + w,
			);
		}
		res
	}
}

impl<T: Number> Mul<Vec4<T>> for Mat3x4<T> {
	type Output = Vec4<T>;

	fn mul(self, v: Vec4<T>) -> Self::Output {
		Vec4::new_with(
			self.rows[0].x * v.x + self.rows[0].y * v.y + self.rows[0].z * v.z + self.rows[0].w * v.w,
			self.rows[1].x * v.x + self.rows[1].y * v.y + self.rows[1].z * v.z + self.rows[1].w * v.w,
			self.rows[2].x * v.x + self.rows[2].y * v.y + self.rows[2].z * v.z + self.rows[2].w * v.w,
			v.w,
		)
	}
}

impl<T: Number> Mul<Vec3<T>> for Mat3x4<T> {
	type Output = Vec3<T>;

	fn mul(self, v: Vec3<T>) -> Self::Output {
		Vec3::new_with(
			self.rows[0].x * v.x + self.rows[0].y * v.y + self.rows[0].z * v.z + self.rows[0].w,
			self.rows[1].x * v.x + self.rows[1].y * v.y + self.rows[1].z * v.z + self.rows[1].w,
			self.rows[2].x * v.x + self.rows[2].y * v.y + self.rows[2].z * v.z + self.rows[2].w,
		)
	}
}

impl<T: Number> From<[Vec4<T>; 3]> for Mat3x4<T> {
	fn from(rows: [Vec4<T>; 3]) -> Self {
		Self { rows }
	}
}

impl<T: Number> From<[T; 12]> for Mat3x4<T> {
	fn from(arr: [T; 12]) -> Self {
		Self {
			rows: [
				Vec4::new(arr[0], arr[1], arr[2], arr[3]),
				Vec4::new(arr[4], arr[5], arr[6], arr[7]),
				Vec4::new(arr[8], arr[9], arr[10], arr[11]),
			],
		}
	}
}

impl<T: Number> From<Mat3x4<T>> for [T; 12] {
	fn from(m: Mat3x4<T>) -> Self {
		m.to_array()
	}
}

impl<T: Number> From<Mat4x4<T>> for Mat3x4<T> {
	fn from(m: Mat4x4<T>) -> Self {
		Self {
			rows: [m.rows[0], m.rows[1], m.rows[2]],
		}
	}
}
