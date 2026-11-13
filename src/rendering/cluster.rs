/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::rhi::{self, AllocationCreate, AllocationLocation, BufferUsage};


static CLUSTER_BUILD_COMP_SPV: &[u8] = include_bytes!("../../res/shaders/cluster_build.comp.spv");
static CLUSTER_CULL_COMP_SPV: &[u8] = include_bytes!("../../res/shaders/cluster_cull.comp.spv");

pub const CLUSTER_DIM_X: u32 = 16;
pub const CLUSTER_DIM_Y: u32 = 9;
pub const CLUSTER_DIM_Z: u32 = 24;
pub const TOTAL_CLUSTERS: u32 = CLUSTER_DIM_X * CLUSTER_DIM_Y * CLUSTER_DIM_Z;
pub const MAX_POINT_LIGHTS_PER_CLUSTER: u32 = 128;
pub const MAX_SPOT_LIGHTS_PER_CLUSTER: u32 = 64;


#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default)]
pub struct GpuClusterAABB {
	pub min_point: [f32; 4],
	pub max_point: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct GpuClusterRecord {
	pub point_offset: u32,
	pub point_count: u32,
	pub spot_offset: u32,
	pub spot_count: u32,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct BuildPushConstants {
	pub inv_proj: [f32; 16],
	pub aabb_buffer_address: u64,
	pub z_near: f32,
	pub z_far: f32,
	pub dim_x: u32,
	pub dim_y: u32,
	pub dim_z: u32,
	pub _pad: u32,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct CullPushConstants {
	pub scene_address: u64,
	pub aabb_buffer_address: u64,
	pub grid_buffer_address: u64,
	pub point_index_list_address: u64,
	pub spot_index_list_address: u64,
	pub max_point_lights: u32,
	pub max_spot_lights: u32,
}


pub struct ClusterManager {
	aabb_buffer: rhi::Buffer,
	grid_buffer: rhi::Buffer,
	point_index_list_buffer: rhi::Buffer,
	spot_index_list_buffer: rhi::Buffer,
	build_pipeline: rhi::Pipeline,
	_build_meta: rhi::PipelineMeta,
	cull_pipeline: rhi::Pipeline,
	_cull_meta: rhi::PipelineMeta,
	last_proj: [f32; 16],
	needs_rebuild: bool,
}

impl ClusterManager {
	pub fn new(device: &rhi::Device, alloc: &rhi::Allocator) -> Result<Self, String> {
		let aabb_size = (TOTAL_CLUSTERS as usize * std::mem::size_of::<GpuClusterAABB>()) as u64;
		let grid_size = (TOTAL_CLUSTERS as usize * std::mem::size_of::<GpuClusterRecord>()) as u64;
		let pt_index_size = (TOTAL_CLUSTERS as usize
			* MAX_POINT_LIGHTS_PER_CLUSTER as usize
			* std::mem::size_of::<u32>()) as u64;
		let spot_index_size = (TOTAL_CLUSTERS as usize
			* MAX_SPOT_LIGHTS_PER_CLUSTER as usize
			* std::mem::size_of::<u32>()) as u64;

		let buffer_usage = BufferUsage::StorageBuffer | BufferUsage::DeviceAddress;

		let aabb_buffer = alloc.alloc_buffer(
			aabb_size,
			buffer_usage,
			AllocationCreate::Dedicated,
			AllocationLocation::PreferDevice,
		);

		let grid_buffer = alloc.alloc_buffer(
			grid_size,
			buffer_usage,
			AllocationCreate::Dedicated,
			AllocationLocation::PreferDevice,
		);

		let point_index_list_buffer = alloc.alloc_buffer(
			pt_index_size,
			buffer_usage,
			AllocationCreate::Dedicated,
			AllocationLocation::PreferDevice,
		);

		let spot_index_list_buffer = alloc.alloc_buffer(
			spot_index_size,
			buffer_usage,
			AllocationCreate::Dedicated,
			AllocationLocation::PreferDevice,
		);

		let build_shader = rhi::Shader::new(
			device,
			rhi::ShaderStage::Compute,
			CLUSTER_BUILD_COMP_SPV,
			"main",
		);

		let build_meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Compute,
			std::mem::size_of::<BuildPushConstants>() as u32,
		);

		let build_pipeline = rhi::Pipeline::new_compute(device, &build_meta, &build_shader);

		let cull_shader = rhi::Shader::new(
			device,
			rhi::ShaderStage::Compute,
			CLUSTER_CULL_COMP_SPV,
			"main",
		);

		let cull_meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Compute,
			std::mem::size_of::<CullPushConstants>() as u32,
		);

		let cull_pipeline = rhi::Pipeline::new_compute(device, &cull_meta, &cull_shader);

		Ok(Self {
			aabb_buffer,
			grid_buffer,
			point_index_list_buffer,
			spot_index_list_buffer,
			build_pipeline,
			_build_meta: build_meta,
			cull_pipeline,
			_cull_meta: cull_meta,
			last_proj: [0.0; 16],
			needs_rebuild: true,
		})
	}

	pub fn grid_address(&self) -> u64 {
		self.grid_buffer.address()
	}

	pub fn point_index_list_address(&self) -> u64 {
		self.point_index_list_buffer.address()
	}

	pub fn spot_index_list_address(&self) -> u64 {
		self.spot_index_list_buffer.address()
	}

	pub fn update_camera(
		&mut self,
		cmd: &mut rhi::Command,
		inv_proj: [f32; 16],
		z_near: f32,
		z_far: f32,
	) {
		if !self.needs_rebuild && self.last_proj == inv_proj {
			return;
		}

		self.last_proj = inv_proj;
		self.needs_rebuild = false;

		let pc = BuildPushConstants {
			inv_proj,
			aabb_buffer_address: self.aabb_buffer.address(),
			z_near,
			z_far,
			dim_x: CLUSTER_DIM_X,
			dim_y: CLUSTER_DIM_Y,
			dim_z: CLUSTER_DIM_Z,
			_pad: 0,
		};

		cmd.bind_pipeline(&self.build_pipeline);
		cmd.push_constant(rhi::ShaderStage::Compute, 0, &pc);
		cmd.dispatch(1, 1, CLUSTER_DIM_Z);
		cmd.compute_barrier();
	}

	pub fn cull_lights(&mut self, cmd: &mut rhi::Command, scene_address: u64) {
		let pc = CullPushConstants {
			scene_address,
			aabb_buffer_address: self.aabb_buffer.address(),
			grid_buffer_address: self.grid_buffer.address(),
			point_index_list_address: self.point_index_list_buffer.address(),
			spot_index_list_address: self.spot_index_list_buffer.address(),
			max_point_lights: MAX_POINT_LIGHTS_PER_CLUSTER,
			max_spot_lights: MAX_SPOT_LIGHTS_PER_CLUSTER,
		};

		cmd.bind_pipeline(&self.cull_pipeline);
		cmd.push_constant(rhi::ShaderStage::Compute, 0, &pc);
		cmd.dispatch(1, 1, CLUSTER_DIM_Z);
		cmd.compute_barrier();
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_cluster_dimensions_and_sizes() {
		assert_eq!(TOTAL_CLUSTERS, 16 * 9 * 24);
		assert_eq!(TOTAL_CLUSTERS, 3456);
		assert_eq!(std::mem::size_of::<GpuClusterAABB>(), 32);
		assert_eq!(std::mem::size_of::<GpuClusterRecord>(), 16);
		assert!(std::mem::size_of::<BuildPushConstants>() <= 128);
		assert!(std::mem::size_of::<CullPushConstants>() <= 128);
	}

	#[test]
	fn test_depth_slicing_math() {
		let z_near = 0.1f32;
		let z_far = 1000.0f32;
		let dim_z = CLUSTER_DIM_Z as f32;

		// Verify depth slice boundaries are monotonic and cover [z_near, z_far]
		let mut prev_z = z_near;
		for slice in 1..=CLUSTER_DIM_Z {
			let slice_z = z_near * (z_far / z_near).powf(slice as f32 / dim_z);
			assert!(slice_z > prev_z, "Depth slicing must be strictly monotonic");
			prev_z = slice_z;
		}
		assert!((prev_z - z_far).abs() < 1e-2, "Last slice must reach z_far");
	}
}
