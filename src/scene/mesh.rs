/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use crate::rhi::{self, AllocationCreate, AllocationLocation, BufferUsage, TransferTime};
use mochi_math::{Vec3, AABB};


#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vertex {
	pub pos: [f32; 4],
	pub normal: [f32; 4],
	pub tangent: [f32; 4],
	pub color: [f32; 4],
	pub uv0: [f32; 2],
	pub uv1: [f32; 2],
}

impl Vertex {
	pub fn new(pos: [f32; 3], normal: [f32; 3], uv: [f32; 2], color: [f32; 4]) -> Self {
		Self {
			pos: [pos[0], pos[1], pos[2], 1.0],
			normal: [normal[0], normal[1], normal[2], 0.0],
			tangent: [1.0, 0.0, 0.0, 1.0],
			color,
			uv0: uv,
			uv1: [0.0, 0.0],
		}
	}
}

pub struct Mesh {
	vertex_buffer: rhi::Buffer,
	index_buffer: Option<rhi::Buffer>,
	vertex_count: u32,
	index_count: u32,
	aabb: AABB<f32>,
}

impl Mesh {
	pub fn new_with_indices(device: &rhi::Device, alloc: &rhi::Allocator, vertices: &[Vertex], indices: &[u32]) -> Arc<Self> {
		let uploader = rhi::Uploader::new(device);

		let mut vertex_buffer = rhi::Buffer::new(
			alloc,
			(std::mem::size_of::<Vertex>() * vertices.len()) as u64,
			BufferUsage::DeviceAddress | BufferUsage::StorageBuffer | BufferUsage::VertexBuffer,
			AllocationCreate::Mapped,
			AllocationLocation::Auto,
		);

		let vertex_bytes = unsafe {
			std::slice::from_raw_parts(
				vertices.as_ptr() as *const u8,
				std::mem::size_of_val(vertices),
			)
		};
		uploader.copy_memory_to_buffer(TransferTime::Now, vertex_bytes, &mut vertex_buffer);

		let mut index_buffer = rhi::Buffer::new(
			alloc,
			(std::mem::size_of::<u32>() * indices.len()) as u64,
			BufferUsage::DeviceAddress | BufferUsage::StorageBuffer | BufferUsage::IndexBuffer,
			AllocationCreate::Mapped,
			AllocationLocation::Auto,
		);

		let index_bytes = unsafe {
			std::slice::from_raw_parts(
				indices.as_ptr() as *const u8,
				std::mem::size_of_val(indices),
			)
		};
		uploader.copy_memory_to_buffer(TransferTime::Now, index_bytes, &mut index_buffer);

		let aabb = Self::compute_aabb(vertices);

		Arc::new(Self {
			vertex_buffer,
			index_buffer: Some(index_buffer),
			vertex_count: vertices.len() as u32,
			index_count: indices.len() as u32,
			aabb,
		})
	}

	pub fn new_unindexed(device: &rhi::Device, alloc: &rhi::Allocator, vertices: &[Vertex]) -> Arc<Self> {
		let uploader = rhi::Uploader::new(device);

		let mut vertex_buffer = rhi::Buffer::new(
			alloc,
			(std::mem::size_of::<Vertex>() * vertices.len()) as u64,
			BufferUsage::DeviceAddress | BufferUsage::StorageBuffer | BufferUsage::VertexBuffer,
			AllocationCreate::Mapped,
			AllocationLocation::Auto,
		);

		let vertex_bytes = unsafe {
			std::slice::from_raw_parts(
				vertices.as_ptr() as *const u8,
				std::mem::size_of_val(vertices),
			)
		};
		uploader.copy_memory_to_buffer(TransferTime::Now, vertex_bytes, &mut vertex_buffer);

		let aabb = Self::compute_aabb(vertices);

		Arc::new(Self {
			vertex_buffer,
			index_buffer: None,
			vertex_count: vertices.len() as u32,
			index_count: 0,
			aabb,
		})
	}

	pub fn vertex_buffer(&self) -> &rhi::Buffer {
		&self.vertex_buffer
	}

	pub fn index_buffer(&self) -> Option<&rhi::Buffer> {
		self.index_buffer.as_ref()
	}

	pub fn vertex_address(&self) -> u64 {
		self.vertex_buffer.address()
	}

	pub fn index_address(&self) -> u64 {
		self.index_buffer.as_ref().map_or(0, |b| b.address())
	}

	pub fn vertex_count(&self) -> u32 {
		self.vertex_count
	}

	pub fn index_count(&self) -> u32 {
		self.index_count
	}

	pub fn is_indexed(&self) -> bool {
		self.index_buffer.is_some()
	}

	pub fn aabb(&self) -> &AABB<f32> {
		&self.aabb
	}

	fn compute_aabb(vertices: &[Vertex]) -> AABB<f32> {
		if vertices.is_empty() {
			return AABB::new(Vec3::ZERO, Vec3::ZERO);
		}
		let mut min = Vec3::new(vertices[0].pos[0], vertices[0].pos[1], vertices[0].pos[2]);
		let mut max = min;
		for v in vertices.iter().skip(1) {
			min.x = min.x.min(v.pos[0]);
			min.y = min.y.min(v.pos[1]);
			min.z = min.z.min(v.pos[2]);
			max.x = max.x.max(v.pos[0]);
			max.y = max.y.max(v.pos[1]);
			max.z = max.z.max(v.pos[2]);
		}
		AABB::new(min, max)
	}

	pub fn create_triangle(device: &rhi::Device, alloc: &rhi::Allocator) -> Arc<Self> {
		let vertices = [
			Vertex::new([0.0, 0.6, 0.0], [0.0, 0.0, 1.0], [0.5, 0.0], [1.0, 0.2, 0.2, 1.0]),
			Vertex::new([-0.6, -0.6, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0], [0.2, 1.0, 0.2, 1.0]),
			Vertex::new([0.6, -0.6, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0], [0.2, 0.4, 1.0, 1.0]),
		];
		Self::new_unindexed(device, alloc, &vertices)
	}

	pub fn create_quad(device: &rhi::Device, alloc: &rhi::Allocator, size: f32) -> Arc<Self> {
		let h = size * 0.5;
		let white = [1.0, 1.0, 1.0, 1.0];
		let norm = [0.0, 0.0, 1.0];
		let vertices = [
			Vertex::new([-h, -h, 0.0], norm, [0.0, 0.0], white),
			Vertex::new([h, -h, 0.0], norm, [1.0, 0.0], white),
			Vertex::new([h, h, 0.0], norm, [1.0, 1.0], white),
			Vertex::new([-h, -h, 0.0], norm, [0.0, 0.0], white),
			Vertex::new([h, h, 0.0], norm, [1.0, 1.0], white),
			Vertex::new([-h, h, 0.0], norm, [0.0, 1.0], white),
		];
		Self::new_unindexed(device, alloc, &vertices)
	}

	pub fn create_cube(device: &rhi::Device, alloc: &rhi::Allocator, size: f32) -> Arc<Self> {
		let h = size * 0.5;

		let red = [0.95, 0.25, 0.25, 1.0];
		let cyan = [0.25, 0.85, 0.95, 1.0];
		let green = [0.25, 0.95, 0.35, 1.0];
		let yellow = [0.95, 0.85, 0.25, 1.0];
		let blue = [0.35, 0.45, 0.95, 1.0];
		let magenta = [0.95, 0.35, 0.85, 1.0];

		let fnz = [0.0, 0.0, 1.0];
		let bnz = [0.0, 0.0, -1.0];
		let tny = [0.0, 1.0, 0.0];
		let bny = [0.0, -1.0, 0.0];
		let rnx = [1.0, 0.0, 0.0];
		let lnx = [-1.0, 0.0, 0.0];

		let vertices = [
			// Front (+Z)
			Vertex::new([-h, -h, h], fnz, [0.0, 0.0], red),
			Vertex::new([h, -h, h], fnz, [1.0, 0.0], red),
			Vertex::new([h, h, h], fnz, [1.0, 1.0], red),
			Vertex::new([-h, -h, h], fnz, [0.0, 0.0], red),
			Vertex::new([h, h, h], fnz, [1.0, 1.0], red),
			Vertex::new([-h, h, h], fnz, [0.0, 1.0], red),

			// Back (-Z)
			Vertex::new([h, -h, -h], bnz, [0.0, 0.0], cyan),
			Vertex::new([-h, -h, -h], bnz, [1.0, 0.0], cyan),
			Vertex::new([-h, h, -h], bnz, [1.0, 1.0], cyan),
			Vertex::new([h, -h, -h], bnz, [0.0, 0.0], cyan),
			Vertex::new([-h, h, -h], bnz, [1.0, 1.0], cyan),
			Vertex::new([h, h, -h], bnz, [0.0, 1.0], cyan),

			// Top (+Y)
			Vertex::new([-h, h, h], tny, [0.0, 0.0], green),
			Vertex::new([h, h, h], tny, [1.0, 0.0], green),
			Vertex::new([h, h, -h], tny, [1.0, 1.0], green),
			Vertex::new([-h, h, h], tny, [0.0, 0.0], green),
			Vertex::new([h, h, -h], tny, [1.0, 1.0], green),
			Vertex::new([-h, h, -h], tny, [0.0, 1.0], green),

			// Bottom (-Y)
			Vertex::new([-h, -h, -h], bny, [0.0, 0.0], yellow),
			Vertex::new([h, -h, -h], bny, [1.0, 0.0], yellow),
			Vertex::new([h, -h, h], bny, [1.0, 1.0], yellow),
			Vertex::new([-h, -h, -h], bny, [0.0, 0.0], yellow),
			Vertex::new([h, -h, h], bny, [1.0, 1.0], yellow),
			Vertex::new([-h, -h, h], bny, [0.0, 1.0], yellow),

			// Right (+X)
			Vertex::new([h, -h, h], rnx, [0.0, 0.0], blue),
			Vertex::new([h, -h, -h], rnx, [1.0, 0.0], blue),
			Vertex::new([h, h, -h], rnx, [1.0, 1.0], blue),
			Vertex::new([h, -h, h], rnx, [0.0, 0.0], blue),
			Vertex::new([h, h, -h], rnx, [1.0, 1.0], blue),
			Vertex::new([h, h, h], rnx, [0.0, 1.0], blue),

			// Left (-X)
			Vertex::new([-h, -h, -h], lnx, [0.0, 0.0], magenta),
			Vertex::new([-h, -h, h], lnx, [1.0, 0.0], magenta),
			Vertex::new([-h, h, h], lnx, [1.0, 1.0], magenta),
			Vertex::new([-h, -h, -h], lnx, [0.0, 0.0], magenta),
			Vertex::new([-h, h, h], lnx, [1.0, 1.0], magenta),
			Vertex::new([-h, h, -h], lnx, [0.0, 1.0], magenta),
		];

		Self::new_unindexed(device, alloc, &vertices)
	}
}
