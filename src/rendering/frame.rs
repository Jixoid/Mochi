/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::rhi::{self, AllocationCreate, AllocationLocation, BufferUsage, TransferTime};
use crate::scene::{CameraComponent, DirectionalLightComponent, PointLightComponent, Scene, TransformComponent};


pub const MAX_DIRECTIONAL_LIGHTS: usize = 4;
pub const MAX_POINT_LIGHTS: usize = 64;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct DirectionalLightGpuData {
	pub direction: [f32; 4],
	pub color: [f32; 4],
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PointLightGpuData {
	pub position: [f32; 4],
	pub color: [f32; 4],
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct SceneFrameData {
	pub view: [f32; 16],
	pub proj: [f32; 16],
	pub view_proj: [f32; 16],
	pub inv_view_proj: [f32; 16],
	pub camera_position: [f32; 4],
	pub ambient_color: [f32; 4],
	pub dir_light_count: u32,
	pub point_light_count: u32,
	pub _pad0: u32,
	pub _pad1: u32,
	pub dir_lights: [DirectionalLightGpuData; MAX_DIRECTIONAL_LIGHTS],
	pub point_lights: [PointLightGpuData; MAX_POINT_LIGHTS],
}

impl Default for SceneFrameData {
	fn default() -> Self {
		let identity_mat = [
			1.0, 0.0, 0.0, 0.0,
			0.0, 1.0, 0.0, 0.0,
			0.0, 0.0, 1.0, 0.0,
			0.0, 0.0, 0.0, 1.0,
		];
		Self {
			view: identity_mat,
			proj: identity_mat,
			view_proj: identity_mat,
			inv_view_proj: identity_mat,
			camera_position: [0.0, 0.0, 0.0, 0.0],
			ambient_color: [0.03, 0.03, 0.03, 0.016],
			dir_light_count: 0,
			point_light_count: 0,
			_pad0: 0,
			_pad1: 0,
			dir_lights: [DirectionalLightGpuData::default(); MAX_DIRECTIONAL_LIGHTS],
			point_lights: [PointLightGpuData::default(); MAX_POINT_LIGHTS],
		}
	}
}

pub struct SceneFrameBuffer {
	gpu_buffer: Option<rhi::Buffer>,
}

impl SceneFrameBuffer {
	pub fn new() -> Self {
		Self { gpu_buffer: None }
	}

	pub fn update(
		&mut self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		scene: &Scene,
		camera_transform: &TransformComponent,
		camera: &CameraComponent,
		aspect: f32,
		time: f32,
		delta_time: f32,
	) {
		let view = camera.view_matrix(camera_transform);
		let proj = camera.projection_matrix(aspect);
		let view_proj = proj * view;
		let inv_view_proj = view_proj.inverse();

		let cam_pos = camera_transform.translation;

		let mut frame_data = SceneFrameData {
			view: view.to_array(),
			proj: proj.to_array(),
			view_proj: view_proj.to_array(),
			inv_view_proj: inv_view_proj.to_array(),
			camera_position: [cam_pos.x, cam_pos.y, cam_pos.z, time],
			ambient_color: [0.15, 0.15, 0.2, delta_time],
			dir_light_count: 0,
			point_light_count: 0,
			_pad0: 0,
			_pad1: 0,
			dir_lights: [DirectionalLightGpuData::default(); MAX_DIRECTIONAL_LIGHTS],
			point_lights: [PointLightGpuData::default(); MAX_POINT_LIGHTS],
		};

		// Query directional lights
		let mut dir_idx = 0;
		for (_e, (t, light)) in scene.world().query::<(&TransformComponent, &DirectionalLightComponent)>().iter() {
			if dir_idx >= MAX_DIRECTIONAL_LIGHTS {
				break;
			}
			let dir = t.rotation * light.direction;
			frame_data.dir_lights[dir_idx] = DirectionalLightGpuData {
				direction: [dir.x, dir.y, dir.z, if light.cast_shadows { 1.0 } else { 0.0 }],
				color: [light.color.x, light.color.y, light.color.z, light.intensity],
			};
			dir_idx += 1;
		}
		frame_data.dir_light_count = dir_idx as u32;

		// Query point lights
		let mut pt_idx = 0;
		for (_e, (t, light)) in scene.world().query::<(&TransformComponent, &PointLightComponent)>().iter() {
			if pt_idx >= MAX_POINT_LIGHTS {
				break;
			}
			let pos = t.translation;
			frame_data.point_lights[pt_idx] = PointLightGpuData {
				position: [pos.x, pos.y, pos.z, light.radius],
				color: [light.color.x, light.color.y, light.color.z, light.intensity],
			};
			pt_idx += 1;
		}
		frame_data.point_light_count = pt_idx as u32;

		let buffer_size = std::mem::size_of::<SceneFrameData>() as u64;

		if self.gpu_buffer.is_none() {
			let buffer = rhi::Buffer::new(
				alloc,
				buffer_size,
				BufferUsage::StorageBuffer | BufferUsage::UniformBuffer | BufferUsage::DeviceAddress,
				AllocationCreate::Mapped,
				AllocationLocation::Auto,
			);
			self.gpu_buffer = Some(buffer);
		}

		if let Some(buffer) = &mut self.gpu_buffer {
			let uploader = rhi::Uploader::new(device);
			let bytes = unsafe {
				std::slice::from_raw_parts(
					&frame_data as *const SceneFrameData as *const u8,
					buffer_size as usize,
				)
			};
			uploader.copy_memory_to_buffer(TransferTime::Now, bytes, buffer);
		}
	}

	pub fn address(&self) -> u64 {
		self.gpu_buffer.as_ref().map_or(0, |b| b.address())
	}
}

impl Default for SceneFrameBuffer {
	fn default() -> Self {
		Self::new()
	}
}
