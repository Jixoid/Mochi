/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use core::ops::Mul;
use crate::{Float, Mat3x4, Mat4x4, Number, Quat, Vec3, Vec4};


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Affine3<T: Number = f32> {
	pub matrix3: [Vec3<T>; 3],
	pub translation: Vec3<T>,
}


impl<T: Number> Affine3<T> {
	pub const ZERO: Self = Self {
		matrix3: [Vec3::ZERO, Vec3::ZERO, Vec3::ZERO],
		translation: Vec3::ZERO,
	};

	pub const IDENTITY: Self = Self {
		matrix3: [
			Vec3::new(T::ONE, T::ZERO, T::ZERO),
			Vec3::new(T::ZERO, T::ONE, T::ZERO),
			Vec3::new(T::ZERO, T::ZERO, T::ONE),
		],
		translation: Vec3::ZERO,
	};


	pub const fn new(matrix3: [Vec3<T>; 3], translation: Vec3<T>) -> Self {
		Self {
			matrix3,
			translation,
		}
	}

	pub const fn from_cols(c0: Vec3<T>, c1: Vec3<T>, c2: Vec3<T>, translation: Vec3<T>) -> Self {
		Self {
			matrix3: [
				Vec3::new(c0.x, c1.x, c2.x),
				Vec3::new(c0.y, c1.y, c2.y),
				Vec3::new(c0.z, c1.z, c2.z),
			],
			translation,
		}
	}

	pub const fn from_translation(translation: Vec3<T>) -> Self {
		Self {
			matrix3: Self::IDENTITY.matrix3,
			translation,
		}
	}

	pub fn from_mat3x4(m: Mat3x4<T>) -> Self {
		Self {
			matrix3: [
				Vec3::new(m.rows[0].x, m.rows[0].y, m.rows[0].z),
				Vec3::new(m.rows[1].x, m.rows[1].y, m.rows[1].z),
				Vec3::new(m.rows[2].x, m.rows[2].y, m.rows[2].z),
			],
			translation: Vec3::new(m.rows[0].w, m.rows[1].w, m.rows[2].w),
		}
	}

	pub fn to_mat3x4(&self) -> Mat3x4<T> {
		Mat3x4::new(
			Vec4::new(self.matrix3[0].x, self.matrix3[0].y, self.matrix3[0].z, self.translation.x),
			Vec4::new(self.matrix3[1].x, self.matrix3[1].y, self.matrix3[1].z, self.translation.y),
			Vec4::new(self.matrix3[2].x, self.matrix3[2].y, self.matrix3[2].z, self.translation.z),
		)
	}

	pub fn to_mat4(&self) -> Mat4x4<T> {
		self.to_mat3x4().to_mat4()
	}

	pub fn transform_point3(&self, p: Vec3<T>) -> Vec3<T> {
		Vec3::new(
			self.matrix3[0].dot(&p) + self.translation.x,
			self.matrix3[1].dot(&p) + self.translation.y,
			self.matrix3[2].dot(&p) + self.translation.z,
		)
	}

	pub fn transform_vector3(&self, v: Vec3<T>) -> Vec3<T> {
		Vec3::new(
			self.matrix3[0].dot(&v),
			self.matrix3[1].dot(&v),
			self.matrix3[2].dot(&v),
		)
	}
}

impl<T: Float> Affine3<T> {
	pub fn from_scale_rotation_translation(scale: Vec3<T>, rotation: Quat<T>, translation: Vec3<T>) -> Self {
		let m = Mat3x4::model(translation, rotation, scale);
		Self::from_mat3x4(m)
	}

	pub fn from_rotation_translation(rotation: Quat<T>, translation: Vec3<T>) -> Self {
		Self::from_scale_rotation_translation(Vec3::ONE, rotation, translation)
	}

	pub fn inverse(&self) -> Self {
		let inv_m = self.to_mat3x4().inverse();
		Self::from_mat3x4(inv_m)
	}
}


impl<T: Number> Default for Affine3<T> {
	fn default() -> Self {
		Self::IDENTITY
	}
}

impl<T: Number> Mul<Self> for Affine3<T> {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		let m = self.to_mat3x4() * rhs.to_mat3x4();
		Self::from_mat3x4(m)
	}
}

impl<T: Number> Mul<Vec3<T>> for Affine3<T> {
	type Output = Vec3<T>;

	fn mul(self, v: Vec3<T>) -> Self::Output {
		self.transform_point3(v)
	}
}

impl<T: Number> From<Mat3x4<T>> for Affine3<T> {
	fn from(m: Mat3x4<T>) -> Self {
		Self::from_mat3x4(m)
	}
}

impl<T: Number> From<Affine3<T>> for Mat3x4<T> {
	fn from(a: Affine3<T>) -> Self {
		a.to_mat3x4()
	}
}

impl<T: Number> From<Affine3<T>> for Mat4x4<T> {
	fn from(a: Affine3<T>) -> Self {
		a.to_mat4()
	}
}

impl<T: Number> From<Vec3<T>> for Affine3<T> {
	fn from(translation: Vec3<T>) -> Self {
		Self::from_translation(translation)
	}
}

impl<T: Float> From<Quat<T>> for Affine3<T> {
	fn from(rotation: Quat<T>) -> Self {
		Self::from_rotation_translation(rotation, Vec3::ZERO)
	}
}
