/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use crate::material::MaterialInstance;
use crate::scene::mesh::Mesh;
use mochi_math::{Mat4x4, Transform, Vec3, Vec4};


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TransformComponent(pub Transform<f32>);

impl Default for TransformComponent {
	fn default() -> Self {
		Self(Transform::IDENTITY)
	}
}

impl Deref for TransformComponent {
	type Target = Transform<f32>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for TransformComponent {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl TransformComponent {
	pub fn new(translation: Vec3<f32>) -> Self {
		let mut t = Transform::IDENTITY;
		t.translation = translation;
		Self(t)
	}

	pub fn matrix(&self) -> Mat4x4<f32> {
		self.0.to_mat4()
	}

	pub fn to_array(&self) -> [f32; 16] {
		self.0.to_mat4().to_array()
	}
}

#[derive(Copy, Clone, Debug)]
pub struct CameraComponent {
	pub fov_deg: f32,
	pub near: f32,
	pub far: f32,
	pub is_active: bool,
	pub clear_color: Vec4<f32>,
}

impl Default for CameraComponent {
	fn default() -> Self {
		Self {
			fov_deg: 60.0,
			near: 0.1,
			far: 1000.0,
			is_active: true,
			clear_color: Vec4::new_with(0.1, 0.15, 0.25, 1.0),
		}
	}
}

impl CameraComponent {
	pub fn new(fov_deg: f32, near: f32, far: f32) -> Self {
		Self {
			fov_deg,
			near,
			far,
			is_active: true,
			clear_color: Vec4::new_with(0.1, 0.15, 0.25, 1.0),
		}
	}

	pub fn view_matrix(&self, transform: &Transform<f32>) -> Mat4x4<f32> {
		let eye = transform.translation;
		let forward = transform.rotation * Vec3::new(0.0, 0.0, -1.0);
		let up = transform.rotation * Vec3::new(0.0, 1.0, 0.0);
		Mat4x4::look_at(eye, eye + forward, up)
	}

	pub fn projection_matrix(&self, aspect: f32) -> Mat4x4<f32> {
		Mat4x4::perspective(self.fov_deg, aspect, self.near, self.far)
	}
}

#[derive(Copy, Clone, Debug)]
pub struct DirectionalLightComponent {
	pub color: Vec3<f32>,
	pub intensity: f32,
	pub direction: Vec3<f32>,
	pub cast_shadows: bool,
}

impl Default for DirectionalLightComponent {
	fn default() -> Self {
		Self {
			color: Vec3::ONE,
			intensity: 1.0,
			direction: Vec3::new(0.5, -1.0, 0.5).normalize(),
			cast_shadows: true,
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub struct PointLightComponent {
	pub color: Vec3<f32>,
	pub intensity: f32,
	pub radius: f32,
}

impl Default for PointLightComponent {
	fn default() -> Self {
		Self {
			color: Vec3::ONE,
			intensity: 1.0,
			radius: 10.0,
		}
	}
}

pub struct MeshRendererComponent {
	pub mesh: Arc<Mesh>,
	pub material: Arc<MaterialInstance>,
	pub cast_shadows: bool,
	pub receive_shadows: bool,
	pub is_visible: bool,
}

impl MeshRendererComponent {
	pub fn new(mesh: Arc<Mesh>, material: Arc<MaterialInstance>) -> Self {
		Self {
			mesh,
			material,
			cast_shadows: true,
			receive_shadows: true,
			is_visible: true,
		}
	}
}
