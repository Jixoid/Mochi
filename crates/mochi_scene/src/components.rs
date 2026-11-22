/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use mochi_material::MaterialInstance;
use crate::mesh::Mesh;
use mochi_math::{Mat3x4, Mat4x4, Transform, Vec3, Vec4};
pub use mochi_phi::{
	BodyDef, BodyHandle, BodyType, ColliderDef, ColliderHandle, ColliderShape, JointHandle,
	PhysicsMaterial, SpringJointDef,
};


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

#[derive(Copy, Clone, Debug)]
pub struct SpotLightComponent {
	pub color: Vec3<f32>,
	pub intensity: f32,
	pub radius: f32,
	pub inner_cone_angle_deg: f32,
	pub outer_cone_angle_deg: f32,
	pub cast_shadows: bool,
}

impl Default for SpotLightComponent {
	fn default() -> Self {
		Self {
			color: Vec3::ONE,
			intensity: 1.0,
			radius: 10.0,
			inner_cone_angle_deg: 25.0,
			outer_cone_angle_deg: 35.0,
			cast_shadows: true,
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

pub struct SkeletonComponent {
	pub joint_matrices: Vec<Mat3x4<f32>>,
	pub joint_buffer: Option<mochi_rhi::Buffer>,
}

impl SkeletonComponent {
	pub fn new(joint_count: usize) -> Self {
		Self {
			joint_matrices: vec![Mat3x4::IDENTITY; joint_count],
			joint_buffer: None,
		}
	}

	pub fn from_matrices(matrices: Vec<Mat3x4<f32>>) -> Self {
		Self {
			joint_matrices: matrices,
			joint_buffer: None,
		}
	}

	pub fn update_gpu(&mut self, device: &mochi_rhi::Device, alloc: &mochi_rhi::Allocator) {
		if self.joint_matrices.is_empty() {
			return;
		}

		let byte_size = (self.joint_matrices.len() * std::mem::size_of::<Mat3x4<f32>>()) as u64;

		let needs_realloc = match &self.joint_buffer {
			Some(buf) => buf.size() < byte_size,
			None => true,
		};

		if needs_realloc {
			self.joint_buffer = Some(mochi_rhi::Buffer::new(
				alloc,
				byte_size.max(256),
				mochi_rhi::BufferUsage::StorageBuffer | mochi_rhi::BufferUsage::DeviceAddress,
				mochi_rhi::AllocationCreate::Mapped,
				mochi_rhi::AllocationLocation::Auto,
			));
		}

		if let Some(buf) = &mut self.joint_buffer {
			let uploader = mochi_rhi::Uploader::new(device);
			let bytes = unsafe {
				std::slice::from_raw_parts(
					self.joint_matrices.as_ptr() as *const u8,
					self.joint_matrices.len() * std::mem::size_of::<Mat3x4<f32>>(),
				)
			};
			uploader.copy_memory_to_buffer(mochi_rhi::TransferTime::Now, bytes, buf);
		}
	}

	pub fn joint_buffer_address(&self) -> u64 {
		self.joint_buffer.as_ref().map_or(0, |b| b.address())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct RigidBodyComponent {
	pub def: BodyDef,
	pub handle: BodyHandle,
}

impl Default for RigidBodyComponent {
	fn default() -> Self {
		Self {
			def: BodyDef::default(),
			handle: BodyHandle::INVALID,
		}
	}
}

impl RigidBodyComponent {
	pub fn new(def: BodyDef) -> Self {
		Self {
			def,
			handle: BodyHandle::INVALID,
		}
	}

	pub fn dynamic() -> Self {
		Self::new(BodyDef::dynamic())
	}

	pub fn static_body() -> Self {
		Self::new(BodyDef::static_body())
	}

	pub fn kinematic() -> Self {
		Self::new(BodyDef::kinematic())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColliderComponent {
	pub def: ColliderDef,
	pub handle: ColliderHandle,
}

impl Default for ColliderComponent {
	fn default() -> Self {
		Self {
			def: ColliderDef::default(),
			handle: ColliderHandle::INVALID,
		}
	}
}

impl ColliderComponent {
	pub fn new(def: ColliderDef) -> Self {
		Self {
			def,
			handle: ColliderHandle::INVALID,
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NameComponent(pub String);

impl NameComponent {
	pub fn new(name: impl Into<String>) -> Self {
		Self(name.into())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpringJointComponent {
	pub target_entity_name: String,
	pub local_anchor_a: Vec3<f32>,
	pub local_anchor_b: Vec3<f32>,
	pub rest_length: f32,
	pub stiffness: f32,
	pub damping: f32,
	pub handle: JointHandle,
}

impl SpringJointComponent {
	pub fn new(target_entity_name: impl Into<String>, rest_length: f32, stiffness: f32, damping: f32) -> Self {
		Self {
			target_entity_name: target_entity_name.into(),
			local_anchor_a: Vec3::ZERO,
			local_anchor_b: Vec3::ZERO,
			rest_length,
			stiffness,
			damping,
			handle: JointHandle::INVALID,
		}
	}

	pub fn with_anchors(mut self, anchor_a: Vec3<f32>, anchor_b: Vec3<f32>) -> Self {
		self.local_anchor_a = anchor_a;
		self.local_anchor_b = anchor_b;
		self
	}
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpringJointsComponent {
	pub joints: Vec<SpringJointComponent>,
}

impl SpringJointsComponent {
	pub fn new(joints: Vec<SpringJointComponent>) -> Self {
		Self { joints }
	}

	pub fn single(joint: SpringJointComponent) -> Self {
		Self {
			joints: vec![joint],
		}
	}
}
