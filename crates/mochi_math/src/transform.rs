/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use core::ops::Mul;
use crate::{Affine3, Float, Mat3x4, Mat4x4, Quat, Vec3};


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transform<T: Float = f32> {
	pub translation: Vec3<T>,
	pub rotation: Quat<T>,
	pub scale: Vec3<T>,
}


impl<T: Float> Transform<T> {
	pub const IDENTITY: Self = Self {translation: Vec3::ZERO, rotation: Quat::IDENTITY, scale: Vec3::ONE};

	pub const fn new(translation: Vec3<T>, rotation: Quat<T>, scale: Vec3<T>) -> Self {
		Self {
			translation,
			rotation,
			scale,
		}
	}


	pub const fn from_translation(translation: Vec3<T>) -> Self {
		Self {
			translation,
			rotation: Quat::IDENTITY,
			scale: Vec3::ONE,
		}
	}

	pub const fn from_rotation(rotation: Quat<T>) -> Self {
		Self {
			translation: Vec3::ZERO,
			rotation,
			scale: Vec3::ONE,
		}
	}

	pub const fn from_scale(scale: Vec3<T>) -> Self {
		Self {
			translation: Vec3::ZERO,
			rotation: Quat::IDENTITY,
			scale,
		}
	}

	pub fn to_mat3x4(&self) -> Mat3x4<T> {
		Mat3x4::model(self.translation, self.rotation, self.scale)
	}

	pub fn to_mat4(&self) -> Mat4x4<T> {
		self.to_mat3x4().to_mat4()
	}

	pub fn to_affine(&self) -> Affine3<T> {
		Affine3::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
	}

	pub fn transform_point(&self, p: Vec3<T>) -> Vec3<T> {
		self.rotation * (p * self.scale) + self.translation
	}

	pub fn transform_vector(&self, v: Vec3<T>) -> Vec3<T> {
		self.rotation * (v * self.scale)
	}

	pub fn inverse(&self) -> Self {
		let inv_rot = self.rotation.inverse();
		let inv_scale = Vec3::new(
			T::ONE / self.scale.x,
			T::ONE / self.scale.y,
			T::ONE / self.scale.z,
		);
		let inv_trans = inv_rot * (-self.translation * inv_scale);

		Self {
			translation: inv_trans,
			rotation: inv_rot,
			scale: inv_scale,
		}
	}

	pub fn looking_at(mut self, target: Vec3<T>, up: Vec3<T>) -> Self {
		self.rotation = Quat::look_at(self.translation, target, up);
		self
	}
}


impl<T: Float> From<Transform<T>> for Mat3x4<T> {
	fn from(t: Transform<T>) -> Self {
		t.to_mat3x4()
	}
}

impl<T: Float> From<Transform<T>> for Mat4x4<T> {
	fn from(t: Transform<T>) -> Self {
		t.to_mat4()
	}
}

impl<T: Float> From<Transform<T>> for Affine3<T> {
	fn from(t: Transform<T>) -> Self {
		t.to_affine()
	}
}

impl<T: Float> From<Vec3<T>> for Transform<T> {
	fn from(translation: Vec3<T>) -> Self {
		Self::from_translation(translation)
	}
}

impl<T: Float> From<Quat<T>> for Transform<T> {
	fn from(rotation: Quat<T>) -> Self {
		Self::from_rotation(rotation)
	}
}

impl<T: Float> From<(Vec3<T>, Quat<T>)> for Transform<T> {
	fn from((translation, rotation): (Vec3<T>, Quat<T>)) -> Self {
		Self::new(translation, rotation, Vec3::ONE)
	}
}

impl<T: Float> From<(Vec3<T>, Quat<T>, Vec3<T>)> for Transform<T> {
	fn from((translation, rotation, scale): (Vec3<T>, Quat<T>, Vec3<T>)) -> Self {
		Self::new(translation, rotation, scale)
	}
}


impl<T: Float> Default for Transform<T> {
	fn default() -> Self {
		Self::IDENTITY
	}
}


impl<T: Float> Mul<Self> for Transform<T> {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		Self {
			translation: self.transform_point(rhs.translation),
			rotation: self.rotation * rhs.rotation,
			scale: self.scale * rhs.scale,
		}
	}
}

impl<T: Float> Mul<Vec3<T>> for Transform<T> {
	type Output = Vec3<T>;

	fn mul(self, v: Vec3<T>) -> Self::Output {
		self.transform_point(v)
	}
}
