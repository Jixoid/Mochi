/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use mochi_rhi::{self as rhi, BufferUsage};
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
	pub joints: [u32; 4],
	pub weights: [f32; 4],
}

const _: () = {
	assert!(std::mem::size_of::<Vertex>() == 112);
};

impl Vertex {
	pub fn new(pos: [f32; 3], normal: [f32; 3], uv: [f32; 2], color: [f32; 4]) -> Self {
		Self {
			pos: [pos[0], pos[1], pos[2], 1.0],
			normal: [normal[0], normal[1], normal[2], 0.0],
			tangent: [1.0, 0.0, 0.0, 1.0],
			color,
			uv0: uv,
			uv1: [0.0, 0.0],
			joints: [0, 0, 0, 0],
			weights: [0.0, 0.0, 0.0, 0.0],
		}
	}

	pub fn new_skinned(
		pos: [f32; 3],
		normal: [f32; 3],
		uv: [f32; 2],
		color: [f32; 4],
		joints: [u32; 4],
		weights: [f32; 4],
	) -> Self {
		Self {
			pos: [pos[0], pos[1], pos[2], 1.0],
			normal: [normal[0], normal[1], normal[2], 0.0],
			tangent: [1.0, 0.0, 0.0, 1.0],
			color,
			uv0: uv,
			uv1: [0.0, 0.0],
			joints,
			weights,
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
	pub fn new_with_indices(
		_device: &rhi::Device,
		alloc: &rhi::Allocator,
		vertices: &[Vertex],
		indices: &[u32],
	) -> Arc<Self> {
		let vertex_buffer = rhi::Buffer::from_slice(
			alloc,
			BufferUsage::DeviceAddress | BufferUsage::StorageBuffer | BufferUsage::VertexBuffer,
			vertices,
		)
		.expect("Failed to create vertex buffer");

		let index_buffer = rhi::Buffer::from_slice(
			alloc,
			BufferUsage::DeviceAddress | BufferUsage::StorageBuffer | BufferUsage::IndexBuffer,
			indices,
		)
		.expect("Failed to create index buffer");

		let aabb = Self::compute_aabb(vertices);

		Arc::new(Self {
			vertex_buffer,
			index_buffer: Some(index_buffer),
			vertex_count: vertices.len() as u32,
			index_count: indices.len() as u32,
			aabb,
		})
	}

	pub fn new_unindexed(
		_device: &rhi::Device,
		alloc: &rhi::Allocator,
		vertices: &[Vertex],
	) -> Arc<Self> {
		let vertex_buffer = rhi::Buffer::from_slice(
			alloc,
			BufferUsage::DeviceAddress | BufferUsage::StorageBuffer | BufferUsage::VertexBuffer,
			vertices,
		)
		.expect("Failed to create vertex buffer");

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
			Vertex::new(
				[0.0, 0.6, 0.0],
				[0.0, 0.0, 1.0],
				[0.5, 0.0],
				[1.0, 0.2, 0.2, 1.0],
			),
			Vertex::new(
				[-0.6, -0.6, 0.0],
				[0.0, 0.0, 1.0],
				[0.0, 1.0],
				[0.2, 1.0, 0.2, 1.0],
			),
			Vertex::new(
				[0.6, -0.6, 0.0],
				[0.0, 0.0, 1.0],
				[1.0, 1.0],
				[0.2, 0.4, 1.0, 1.0],
			),
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

#[cfg(test)]
mod tests {
	use super::*;
	use mochi_math::{Mat3x4, Vec4};
	use crate::components::SkeletonComponent;

	#[test]
	fn test_vertex_layout_and_size() {
		assert_eq!(std::mem::size_of::<Vertex>(), 112);
		let v = Vertex::new_skinned(
			[1.0, 2.0, 3.0],
			[0.0, 1.0, 0.0],
			[0.5, 0.5],
			[1.0, 1.0, 1.0, 1.0],
			[0, 1, 0, 0],
			[0.7, 0.3, 0.0, 0.0],
		);
		assert_eq!(v.joints, [0, 1, 0, 0]);
		assert_eq!(v.weights, [0.7, 0.3, 0.0, 0.0]);
	}

	#[test]
	fn test_skeleton_component_affine3_matrices() {
		let mut skel = SkeletonComponent::new(4);
		assert_eq!(skel.joint_matrices.len(), 4);
		assert_eq!(skel.joint_matrices[0], Mat3x4::IDENTITY);

		// Transform joint 1 with translation [2.0, 0.0, 0.0]
		skel.joint_matrices[1].rows[0].w = 2.0;

		// Skinning math simulation matching pbr.vert / shadow.vert
		let weights = [0.5_f32, 0.5, 0.0, 0.0];
		let joints = [0usize, 1, 0, 0];

		let j0 = skel.joint_matrices[joints[0]];
		let j1 = skel.joint_matrices[joints[1]];

		let r0 = j0.rows[0] * weights[0] + j1.rows[0] * weights[1];
		let r1 = j0.rows[1] * weights[0] + j1.rows[1] * weights[1];
		let r2 = j0.rows[2] * weights[0] + j1.rows[2] * weights[1];

		let p_orig = Vec4::new_with(0.0_f32, 0.0, 0.0, 1.0);
		let p_skinned = mochi_math::Vec3::new(r0.dot(&p_orig), r1.dot(&p_orig), r2.dot(&p_orig));

		// 0.5 * 0.0 + 0.5 * 2.0 = 1.0
		assert!(
			(p_skinned.x - 1.0).abs() < 1e-5,
			"Expected x=1.0, got {}",
			p_skinned.x
		);
		assert!(
			(p_skinned.y - 0.0).abs() < 1e-5,
			"Expected y=0.0, got {}",
			p_skinned.y
		);
		assert!(
			(p_skinned.z - 0.0).abs() < 1e-5,
			"Expected z=0.0, got {}",
			p_skinned.z
		);
	}
}
