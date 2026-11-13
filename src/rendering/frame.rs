/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::rendering::cluster::{
	CLUSTER_DIM_X, CLUSTER_DIM_Y, CLUSTER_DIM_Z, MAX_POINT_LIGHTS_PER_CLUSTER,
};
use crate::rhi::{self, AllocationCreate, AllocationLocation, BufferUsage, TransferTime};
use crate::scene::{
	CameraComponent, DirectionalLightComponent, PointLightComponent, Scene, SpotLightComponent,
	TransformComponent,
};


pub const MAX_DIRECTIONAL_LIGHTS: usize = 4;

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
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct SpotLightGpuData {
	pub position_radius: [f32; 4],
	pub color_intensity: [f32; 4],
	pub direction_cos_inner: [f32; 4],
	pub params: [f32; 4],
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
	pub cluster_grid_address: u64,
	pub point_light_index_list_address: u64,
	pub spot_light_index_list_address: u64,
	pub point_lights_address: u64,
	pub spot_lights_address: u64,
	pub cluster_screen_size: [f32; 2],
	pub z_near: f32,
	pub z_far: f32,
	pub cluster_dims: [u32; 4],
	pub dir_light_count: u32,
	pub point_light_count: u32,
	pub spot_light_count: u32,
	pub _pad0: u32,
	pub _pad1: [u32; 2],
	pub dir_lights: [DirectionalLightGpuData; MAX_DIRECTIONAL_LIGHTS],
	pub cascade_view_proj: [[[f32; 16]; 4]; MAX_DIRECTIONAL_LIGHTS],
	pub cascade_split_depths: [f32; 4],
	pub cascade_shadow_ids: [[u32; 4]; MAX_DIRECTIONAL_LIGHTS],
	pub prev_view_proj: [f32; 16],
	pub jitter: [f32; 4],
}

const _: () = {
	assert!(std::mem::size_of::<SceneFrameData>() == 1696);
	assert!(std::mem::offset_of!(SceneFrameData, cluster_grid_address) == 288);
	assert!(std::mem::offset_of!(SceneFrameData, point_light_index_list_address) == 296);
	assert!(std::mem::offset_of!(SceneFrameData, spot_light_index_list_address) == 304);
	assert!(std::mem::offset_of!(SceneFrameData, point_lights_address) == 312);
	assert!(std::mem::offset_of!(SceneFrameData, spot_lights_address) == 320);
	assert!(std::mem::offset_of!(SceneFrameData, cluster_screen_size) == 328);
	assert!(std::mem::offset_of!(SceneFrameData, z_near) == 336);
	assert!(std::mem::offset_of!(SceneFrameData, z_far) == 340);
	assert!(std::mem::offset_of!(SceneFrameData, cluster_dims) == 344);
	assert!(std::mem::offset_of!(SceneFrameData, dir_light_count) == 360);
	assert!(std::mem::offset_of!(SceneFrameData, point_light_count) == 364);
	assert!(std::mem::offset_of!(SceneFrameData, spot_light_count) == 368);
	assert!(std::mem::offset_of!(SceneFrameData, dir_lights) == 384);
	assert!(std::mem::offset_of!(SceneFrameData, cascade_view_proj) == 512);
	assert!(std::mem::offset_of!(SceneFrameData, cascade_split_depths) == 1536);
	assert!(std::mem::offset_of!(SceneFrameData, cascade_shadow_ids) == 1552);
	assert!(std::mem::offset_of!(SceneFrameData, prev_view_proj) == 1616);
	assert!(std::mem::offset_of!(SceneFrameData, jitter) == 1680);
};

impl Default for SceneFrameData {
	fn default() -> Self {
		let identity_mat = [
			1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
		];
		Self {
			view: identity_mat,
			proj: identity_mat,
			view_proj: identity_mat,
			inv_view_proj: identity_mat,
			camera_position: [0.0, 0.0, 0.0, 0.0],
			ambient_color: [0.03, 0.03, 0.03, 0.016],
			cluster_grid_address: 0,
			point_light_index_list_address: 0,
			spot_light_index_list_address: 0,
			point_lights_address: 0,
			spot_lights_address: 0,
			cluster_screen_size: [1.0, 1.0],
			z_near: 0.1,
			z_far: 1000.0,
			cluster_dims: [
				CLUSTER_DIM_X,
				CLUSTER_DIM_Y,
				CLUSTER_DIM_Z,
				MAX_POINT_LIGHTS_PER_CLUSTER,
			],
			dir_light_count: 0,
			point_light_count: 0,
			spot_light_count: 0,
			_pad0: 0,
			_pad1: [0; 2],
			dir_lights: [DirectionalLightGpuData::default(); MAX_DIRECTIONAL_LIGHTS],
			cascade_view_proj: [[identity_mat; 4]; MAX_DIRECTIONAL_LIGHTS],
			cascade_split_depths: [2.5, 8.0, 24.0, 80.0],
			cascade_shadow_ids: [[0; 4]; MAX_DIRECTIONAL_LIGHTS],
			prev_view_proj: identity_mat,
			jitter: [0.0; 4],
		}
	}
}

pub struct SceneFrameBuffer {
	gpu_buffer: Option<rhi::Buffer>,
	point_light_buffer: Option<rhi::Buffer>,
	point_light_capacity: usize,
	spot_light_buffer: Option<rhi::Buffer>,
	spot_light_capacity: usize,
	point_light_count: u32,
	spot_light_count: u32,
	primary_sun_dir: mochi_math::Vec3<f32>,
	prev_view_proj: Option<[f32; 16]>,
	prev_jitter: [f32; 2],
}

impl SceneFrameBuffer {
	pub fn new() -> Self {
		Self {
			gpu_buffer: None,
			point_light_buffer: None,
			point_light_capacity: 0,
			spot_light_buffer: None,
			spot_light_capacity: 0,
			point_light_count: 0,
			spot_light_count: 0,
			primary_sun_dir: mochi_math::Vec3::new(0.5, -1.0, 0.3).normalize(),
			prev_view_proj: None,
			prev_jitter: [0.0; 2],
		}
	}

	#[allow(clippy::too_many_arguments)]
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
		cluster_grid_address: u64,
		point_light_index_list_address: u64,
		spot_light_index_list_address: u64,
		screen_width: u32,
		screen_height: u32,
		cascade_view_proj: [[[f32; 16]; 4]; MAX_DIRECTIONAL_LIGHTS],
		cascade_split_depths: [f32; 4],
		cascade_shadow_ids: [[u32; 4]; MAX_DIRECTIONAL_LIGHTS],
		jitter: (f32, f32),
	) {
		let view = camera.view_matrix(camera_transform);
		let mut proj = camera.projection_matrix(aspect);

		let jitter_ndc_x = if jitter.0 != 0.0 {
			2.0 * jitter.0 / screen_width as f32
		} else {
			0.0
		};
		let jitter_ndc_y = if jitter.1 != 0.0 {
			2.0 * jitter.1 / screen_height as f32
		} else {
			0.0
		};

		if jitter_ndc_x != 0.0 || jitter_ndc_y != 0.0 {
			proj.rows[0].z -= jitter_ndc_x;
			proj.rows[1].z -= jitter_ndc_y;
		}

		let view_proj = proj * view;
		let inv_view_proj = view_proj.inverse();

		let prev_vp = self.prev_view_proj.unwrap_or_else(|| view_proj.to_array());
		self.prev_view_proj = Some(view_proj.to_array());

		let jitter_data = [
			jitter_ndc_x,
			jitter_ndc_y,
			self.prev_jitter[0],
			self.prev_jitter[1],
		];
		self.prev_jitter = [jitter_ndc_x, jitter_ndc_y];

		let cam_pos = camera_transform.translation;

		// 1. Query Directional Lights
		let mut dir_lights = [DirectionalLightGpuData::default(); MAX_DIRECTIONAL_LIGHTS];
		let mut dir_idx = 0;
		for (_e, (t, light)) in scene
			.world()
			.query::<(&TransformComponent, &DirectionalLightComponent)>()
			.iter()
		{
			if dir_idx >= MAX_DIRECTIONAL_LIGHTS {
				break;
			}
			let dir = t.rotation * light.direction;
			if dir_idx == 0 {
				self.primary_sun_dir = dir;
			}
			dir_lights[dir_idx] = DirectionalLightGpuData {
				direction: [
					dir.x,
					dir.y,
					dir.z,
					if light.cast_shadows { 1.0 } else { 0.0 },
				],
				color: [light.color.x, light.color.y, light.color.z, light.intensity],
			};
			dir_idx += 1;
		}

		// 2. Query Dynamic Point Lights
		let mut point_lights_vec = Vec::new();
		for (_e, (t, light)) in scene
			.world()
			.query::<(&TransformComponent, &PointLightComponent)>()
			.iter()
		{
			let pos = t.translation;
			point_lights_vec.push(PointLightGpuData {
				position: [pos.x, pos.y, pos.z, light.radius],
				color: [light.color.x, light.color.y, light.color.z, light.intensity],
			});
		}
		self.point_light_count = point_lights_vec.len() as u32;

		if !point_lights_vec.is_empty() {
			let required_cap = point_lights_vec.len().max(64).next_power_of_two();
			if self.point_light_buffer.is_none() || self.point_light_capacity < point_lights_vec.len() {
				let buf_size = (required_cap * std::mem::size_of::<PointLightGpuData>()) as u64;
				self.point_light_buffer = Some(rhi::Buffer::new(
					alloc,
					buf_size,
					BufferUsage::StorageBuffer | BufferUsage::DeviceAddress,
					AllocationCreate::Mapped,
					AllocationLocation::Auto,
				));
				self.point_light_capacity = required_cap;
			}

			if let Some(buf) = &mut self.point_light_buffer {
				let uploader = rhi::Uploader::new(device);
				let bytes = unsafe {
					std::slice::from_raw_parts(
						point_lights_vec.as_ptr() as *const u8,
						point_lights_vec.len() * std::mem::size_of::<PointLightGpuData>(),
					)
				};
				uploader.copy_memory_to_buffer(TransferTime::Now, bytes, buf);
			}
		}

		// 3. Query Dynamic Spot Lights
		let mut spot_lights_vec = Vec::new();
		for (_e, (t, light)) in scene
			.world()
			.query::<(&TransformComponent, &SpotLightComponent)>()
			.iter()
		{
			let pos = t.translation;
			let dir = t.rotation * mochi_math::Vec3::new(0.0, 0.0, -1.0);
			let inner_cos = (light.inner_cone_angle_deg.to_radians() * 0.5).cos();
			let outer_cos = (light.outer_cone_angle_deg.to_radians() * 0.5).cos();

			spot_lights_vec.push(SpotLightGpuData {
				position_radius: [pos.x, pos.y, pos.z, light.radius],
				color_intensity: [light.color.x, light.color.y, light.color.z, light.intensity],
				direction_cos_inner: [dir.x, dir.y, dir.z, inner_cos],
				params: [
					outer_cos,
					if light.cast_shadows { 1.0 } else { 0.0 },
					0.0,
					0.0,
				],
			});
		}
		self.spot_light_count = spot_lights_vec.len() as u32;

		if !spot_lights_vec.is_empty() {
			let required_cap = spot_lights_vec.len().max(32).next_power_of_two();
			if self.spot_light_buffer.is_none() || self.spot_light_capacity < spot_lights_vec.len() {
				let buf_size = (required_cap * std::mem::size_of::<SpotLightGpuData>()) as u64;
				self.spot_light_buffer = Some(rhi::Buffer::new(
					alloc,
					buf_size,
					BufferUsage::StorageBuffer | BufferUsage::DeviceAddress,
					AllocationCreate::Mapped,
					AllocationLocation::Auto,
				));
				self.spot_light_capacity = required_cap;
			}

			if let Some(buf) = &mut self.spot_light_buffer {
				let uploader = rhi::Uploader::new(device);
				let bytes = unsafe {
					std::slice::from_raw_parts(
						spot_lights_vec.as_ptr() as *const u8,
						spot_lights_vec.len() * std::mem::size_of::<SpotLightGpuData>(),
					)
				};
				uploader.copy_memory_to_buffer(TransferTime::Now, bytes, buf);
			}
		}

		let point_lights_address = self.point_light_buffer.as_ref().map_or(0, |b| b.address());
		let spot_lights_address = self.spot_light_buffer.as_ref().map_or(0, |b| b.address());

		let frame_data = SceneFrameData {
			view: view.to_array(),
			proj: proj.to_array(),
			view_proj: view_proj.to_array(),
			inv_view_proj: inv_view_proj.to_array(),
			camera_position: [cam_pos.x, cam_pos.y, cam_pos.z, time],
			ambient_color: [0.15, 0.15, 0.2, delta_time],
			cluster_grid_address,
			point_light_index_list_address,
			spot_light_index_list_address,
			point_lights_address,
			spot_lights_address,
			cluster_screen_size: [screen_width as f32, screen_height as f32],
			z_near: camera.near,
			z_far: camera.far,
			cluster_dims: [
				CLUSTER_DIM_X,
				CLUSTER_DIM_Y,
				CLUSTER_DIM_Z,
				MAX_POINT_LIGHTS_PER_CLUSTER,
			],
			dir_light_count: dir_idx as u32,
			point_light_count: self.point_light_count,
			spot_light_count: self.spot_light_count,
			_pad0: 0,
			_pad1: [0; 2],
			dir_lights,
			cascade_view_proj,
			cascade_split_depths,
			cascade_shadow_ids,
			prev_view_proj: prev_vp,
			jitter: jitter_data,
		};

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

	pub fn point_light_count(&self) -> u32 {
		self.point_light_count
	}

	pub fn spot_light_count(&self) -> u32 {
		self.spot_light_count
	}

	pub fn primary_sun_direction(&self) -> mochi_math::Vec3<f32> {
		self.primary_sun_dir
	}
}

impl Default for SceneFrameBuffer {
	fn default() -> Self {
		Self::new()
	}
}
