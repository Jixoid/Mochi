/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_math::{Mat4x4, Quat, Vec3};
use mochi_rhi::{
	self as rhi, AllocationCreate, AllocationLocation, BufferUsage, TransferTime,
};


static DEBUG_LINE_VERT_SPV: &[u8] = include_bytes!("../../../res/shaders/debug_line.vert.spv");
static DEBUG_LINE_FRAG_SPV: &[u8] = include_bytes!("../../../res/shaders/debug_line.frag.spv");
static DEBUG_BILLBOARD_VERT_SPV: &[u8] =
	include_bytes!("../../../res/shaders/debug_billboard.vert.spv");
static DEBUG_BILLBOARD_FRAG_SPV: &[u8] =
	include_bytes!("../../../res/shaders/debug_billboard.frag.spv");
static DEBUG_GRID_VERT_SPV: &[u8] = include_bytes!("../../../res/shaders/debug_grid.vert.spv");
static DEBUG_GRID_FRAG_SPV: &[u8] = include_bytes!("../../../res/shaders/debug_grid.frag.spv");


#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default)]
pub struct DebugVertex {
	pub position: [f32; 4],
	pub color: [f32; 4],
}

const _: () = {
	assert!(std::mem::size_of::<DebugVertex>() == 32);
};


#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default)]
pub struct DebugBillboard {
	pub position_size: [f32; 4],
	pub color: [f32; 4],
	pub icon_kind: u32,
	pub _pad: [u32; 3],
}

const _: () = {
	assert!(std::mem::size_of::<DebugBillboard>() == 48);
};


#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
struct DebugPushConstants {
	scene_address: u64,
	data_address: u64,
}

const _: () = {
	assert!(std::mem::size_of::<DebugPushConstants>() == 16);
};


#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DebugIconKind {
	Camera = 0,
	DirectionalLight = 1,
	PointLight = 2,
	SpotLight = 3,
}


#[derive(Default)]
pub struct DebugDraw {
	pub draw_grid: bool,
	pub lines: Vec<DebugVertex>,
	pub billboards: Vec<DebugBillboard>,
}

impl DebugDraw {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn clear(&mut self) {
		self.lines.clear();
		self.billboards.clear();
	}

	pub fn is_empty(&self) -> bool {
		self.lines.is_empty() && self.billboards.is_empty()
	}

	pub fn line(&mut self, a: Vec3<f32>, b: Vec3<f32>, color: [f32; 4]) {
		self.lines.push(DebugVertex {
			position: [a.x, a.y, a.z, 1.0],
			color,
		});
		self.lines.push(DebugVertex {
			position: [b.x, b.y, b.z, 1.0],
			color,
		});
	}

	pub fn ray(&mut self, origin: Vec3<f32>, dir: Vec3<f32>, length: f32, color: [f32; 4]) {
		let d = safe_normalize(dir, Vec3::new(0.0, -1.0, 0.0));
		self.line(origin, origin + d * length, color);
	}

	pub fn arrow(
		&mut self,
		origin: Vec3<f32>,
		dir: Vec3<f32>,
		length: f32,
		head_size: f32,
		color: [f32; 4],
	) {
		let d = safe_normalize(dir, Vec3::new(0.0, -1.0, 0.0));
		let tip = origin + d * length;
		self.line(origin, tip, color);

		let (u, v) = ortho_basis(d);
		let back = tip - d * head_size;
		let wing = head_size * 0.45;
		self.line(tip, back + u * wing, color);
		self.line(tip, back - u * wing, color);
		self.line(tip, back + v * wing, color);
		self.line(tip, back - v * wing, color);
	}

	pub fn wire_circle(
		&mut self,
		center: Vec3<f32>,
		axis_u: Vec3<f32>,
		axis_v: Vec3<f32>,
		radius: f32,
		segments: usize,
		color: [f32; 4],
	) {
		let segs = segments.max(6);
		let step = std::f32::consts::TAU / segs as f32;
		for i in 0..segs {
			let a0 = i as f32 * step;
			let a1 = (i + 1) as f32 * step;
			let p0 = center + (axis_u * a0.cos() + axis_v * a0.sin()) * radius;
			let p1 = center + (axis_u * a1.cos() + axis_v * a1.sin()) * radius;
			self.line(p0, p1, color);
		}
	}

	pub fn wire_sphere(&mut self, center: Vec3<f32>, radius: f32, color: [f32; 4]) {
		let r = radius.max(0.001);
		let ux = Vec3::new(1.0, 0.0, 0.0);
		let uy = Vec3::new(0.0, 1.0, 0.0);
		let uz = Vec3::new(0.0, 0.0, 1.0);
		self.wire_circle(center, ux, uy, r, 24, color);
		self.wire_circle(center, ux, uz, r, 24, color);
		self.wire_circle(center, uy, uz, r, 24, color);
	}

	pub fn wire_box(
		&mut self,
		center: Vec3<f32>,
		half_extents: Vec3<f32>,
		rotation: Quat<f32>,
		color: [f32; 4],
	) {
		let corners = [
			Vec3::new(-half_extents.x, -half_extents.y, -half_extents.z),
			Vec3::new(half_extents.x, -half_extents.y, -half_extents.z),
			Vec3::new(half_extents.x, half_extents.y, -half_extents.z),
			Vec3::new(-half_extents.x, half_extents.y, -half_extents.z),
			Vec3::new(-half_extents.x, -half_extents.y, half_extents.z),
			Vec3::new(half_extents.x, -half_extents.y, half_extents.z),
			Vec3::new(half_extents.x, half_extents.y, half_extents.z),
			Vec3::new(-half_extents.x, half_extents.y, half_extents.z),
		]
		.map(|c| center + rotation * c);

		const EDGES: [(usize, usize); 12] = [
			(0, 1),
			(1, 2),
			(2, 3),
			(3, 0),
			(4, 5),
			(5, 6),
			(6, 7),
			(7, 4),
			(0, 4),
			(1, 5),
			(2, 6),
			(3, 7),
		];
		for (i, j) in EDGES {
			self.line(corners[i], corners[j], color);
		}
	}

	pub fn wire_cylinder(
		&mut self,
		center: Vec3<f32>,
		radius: f32,
		height: f32,
		rotation: Quat<f32>,
		color: [f32; 4],
	) {
		let r = radius.max(0.001);
		let hh = height.max(0.001) * 0.5;
		let ux = rotation * Vec3::new(1.0, 0.0, 0.0);
		let uy = rotation * Vec3::new(0.0, 1.0, 0.0);
		let uz = rotation * Vec3::new(0.0, 0.0, 1.0);

		let top = center + uy * hh;
		let bot = center - uy * hh;
		self.wire_circle(top, ux, uz, r, 20, color);
		self.wire_circle(bot, ux, uz, r, 20, color);
		for dir in [ux, -ux, uz, -uz] {
			self.line(top + dir * r, bot + dir * r, color);
		}
	}

	pub fn wire_capsule(
		&mut self,
		center: Vec3<f32>,
		radius: f32,
		height: f32,
		rotation: Quat<f32>,
		color: [f32; 4],
	) {
		let r = radius.max(0.001);
		let hh = height.max(0.0) * 0.5;
		let ux = rotation * Vec3::new(1.0, 0.0, 0.0);
		let uy = rotation * Vec3::new(0.0, 1.0, 0.0);
		let uz = rotation * Vec3::new(0.0, 0.0, 1.0);

		let top = center + uy * hh;
		let bot = center - uy * hh;
		self.wire_circle(top, ux, uz, r, 20, color);
		self.wire_circle(bot, ux, uz, r, 20, color);
		for dir in [ux, -ux, uz, -uz] {
			self.line(top + dir * r, bot + dir * r, color);
		}

		// Hemispherical end caps along X and Z planes
		for (axis_a, axis_b) in [(ux, uy), (uz, uy)] {
			let segs = 12;
			let step = std::f32::consts::PI / segs as f32;
			for i in 0..segs {
				let a0 = i as f32 * step;
				let a1 = (i + 1) as f32 * step;
				let p0 = top + (axis_a * a0.cos() + axis_b * a0.sin()) * r;
				let p1 = top + (axis_a * a1.cos() + axis_b * a1.sin()) * r;
				self.line(p0, p1, color);

				let q0 = bot + (axis_a * a0.cos() - axis_b * a0.sin()) * r;
				let q1 = bot + (axis_a * a1.cos() - axis_b * a1.sin()) * r;
				self.line(q0, q1, color);
			}
		}
	}

	pub fn wire_cone(
		&mut self,
		origin: Vec3<f32>,
		dir: Vec3<f32>,
		length: f32,
		half_angle_deg: f32,
		segments: usize,
		color: [f32; 4],
	) {
		let d = safe_normalize(dir, Vec3::new(0.0, 0.0, -1.0));
		let (u, v) = ortho_basis(d);
		let rad = half_angle_deg.clamp(0.5, 89.0).to_radians();
		let ring_radius = length * rad.tan();
		let center = origin + d * length;

		self.wire_circle(center, u, v, ring_radius, segments, color);
		for s in [u, -u, v, -v] {
			self.line(origin, center + s * ring_radius, color);
		}
	}

	pub fn wire_frustum(&mut self, view_proj_inv: Mat4x4<f32>, color: [f32; 4]) {
		// Vulkan NDC depth is [0, 1]
		let ndc_corners = [
			[-1.0, -1.0, 0.0, 1.0],
			[1.0, -1.0, 0.0, 1.0],
			[1.0, 1.0, 0.0, 1.0],
			[-1.0, 1.0, 0.0, 1.0],
			[-1.0, -1.0, 1.0, 1.0],
			[1.0, -1.0, 1.0, 1.0],
			[1.0, 1.0, 1.0, 1.0],
			[-1.0, 1.0, 1.0, 1.0],
		];
		let world = ndc_corners.map(|c| transform_point_proj(&view_proj_inv, c));

		const EDGES: [(usize, usize); 12] = [
			(0, 1),
			(1, 2),
			(2, 3),
			(3, 0),
			(4, 5),
			(5, 6),
			(6, 7),
			(7, 4),
			(0, 4),
			(1, 5),
			(2, 6),
			(3, 7),
		];
		for (i, j) in EDGES {
			self.line(world[i], world[j], color);
		}
	}

	pub fn billboard(
		&mut self,
		position: Vec3<f32>,
		size: f32,
		color: [f32; 4],
		icon: DebugIconKind,
	) {
		self.billboards.push(DebugBillboard {
			position_size: [position.x, position.y, position.z, size],
			color,
			icon_kind: icon as u32,
			_pad: [0; 3],
		});
	}
}


pub struct DebugDrawRenderer {
	_meta: rhi::PipelineMeta,
	line_pipeline: rhi::Pipeline,
	billboard_pipeline: rhi::Pipeline,
	grid_pipeline: rhi::Pipeline,
	line_buffers: [Option<rhi::Buffer>; 2],
	line_capacities: [usize; 2],
	line_counts: [u32; 2],
	billboard_buffers: [Option<rhi::Buffer>; 2],
	billboard_capacities: [usize; 2],
	billboard_counts: [u32; 2],
}

impl DebugDrawRenderer {
	pub fn new(
		device: &rhi::Device,
		color_format: rhi::Format,
		depth_format: rhi::Format,
		samples: rhi::MsaaSamples,
	) -> Self {
		let meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel,
			std::mem::size_of::<DebugPushConstants>() as u32,
		);

		let line_vert = rhi::Shader::new(device, rhi::ShaderStage::Vertex, DEBUG_LINE_VERT_SPV, "main");
		let line_frag = rhi::Shader::new(device, rhi::ShaderStage::Pixel, DEBUG_LINE_FRAG_SPV, "main");
		let line_pipeline = rhi::GraphicsPipelineBuilder::new(device, &meta)
			.shaders(&[&line_vert, &line_frag])
			.topology(rhi::PrimitiveTopology::LineList)
			.color_format(color_format)
			.depth(depth_format, false)
			.blend(true)
			.samples(samples)
			.build();

		let bb_vert = rhi::Shader::new(
			device,
			rhi::ShaderStage::Vertex,
			DEBUG_BILLBOARD_VERT_SPV,
			"main",
		);
		let bb_frag = rhi::Shader::new(
			device,
			rhi::ShaderStage::Pixel,
			DEBUG_BILLBOARD_FRAG_SPV,
			"main",
		);
		let billboard_pipeline = rhi::GraphicsPipelineBuilder::new(device, &meta)
			.shaders(&[&bb_vert, &bb_frag])
			.topology(rhi::PrimitiveTopology::TriangleList)
			.color_format(color_format)
			.depth(depth_format, false)
			.blend(true)
			.samples(samples)
			.build();

		let grid_vert = rhi::Shader::new(device, rhi::ShaderStage::Vertex, DEBUG_GRID_VERT_SPV, "main");
		let grid_frag = rhi::Shader::new(device, rhi::ShaderStage::Pixel, DEBUG_GRID_FRAG_SPV, "main");
		let grid_pipeline = rhi::GraphicsPipelineBuilder::new(device, &meta)
			.shaders(&[&grid_vert, &grid_frag])
			.topology(rhi::PrimitiveTopology::TriangleList)
			.color_format(color_format)
			.depth(depth_format, false)
			.blend(true)
			.samples(samples)
			.build();

		Self {
			_meta: meta,
			line_pipeline,
			billboard_pipeline,
			grid_pipeline,
			line_buffers: [None, None],
			line_capacities: [0, 0],
			line_counts: [0, 0],
			billboard_buffers: [None, None],
			billboard_capacities: [0, 0],
			billboard_counts: [0, 0],
		}
	}

	pub fn upload(
		&mut self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		frame: usize,
		draw: &DebugDraw,
	) {
		self.line_counts[frame] = draw.lines.len() as u32;
		if !draw.lines.is_empty() {
			Self::ensure_and_upload(
				device,
				alloc,
				&mut self.line_buffers[frame],
				&mut self.line_capacities[frame],
				&draw.lines,
			);
		}

		self.billboard_counts[frame] = draw.billboards.len() as u32;
		if !draw.billboards.is_empty() {
			Self::ensure_and_upload(
				device,
				alloc,
				&mut self.billboard_buffers[frame],
				&mut self.billboard_capacities[frame],
				&draw.billboards,
			);
		}
	}

	fn ensure_and_upload<T: Copy>(
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		buffer: &mut Option<rhi::Buffer>,
		capacity: &mut usize,
		items: &[T],
	) {
		let elem_size = std::mem::size_of::<T>();
		if buffer.is_none() || *capacity < items.len() {
			let new_cap = items.len().max(64).next_power_of_two();
			let buf_size = (new_cap * elem_size) as u64;
			*buffer = Some(rhi::Buffer::new(
				alloc,
				buf_size,
				BufferUsage::StorageBuffer | BufferUsage::DeviceAddress,
				AllocationCreate::Mapped,
				AllocationLocation::Auto,
			));
			*capacity = new_cap;
		}

		if let Some(buf) = buffer.as_mut() {
			let uploader = rhi::Uploader::new(device);
			let bytes = unsafe {
				std::slice::from_raw_parts(items.as_ptr() as *const u8, std::mem::size_of_val(items))
			};
			uploader.copy_memory_to_buffer(TransferTime::Now, bytes, buf);
		}
	}

	pub fn record(
		&self,
		pass: &mut rhi::Command,
		width: u32,
		height: u32,
		scene_addr: u64,
		frame: usize,
		draw_grid: bool,
	) {
		let line_count = self.line_counts[frame];
		let billboard_count = self.billboard_counts[frame];
		if !draw_grid && line_count == 0 && billboard_count == 0 {
			return;
		}

		pass.set_viewport_and_scissor(width, height);

		if draw_grid {
			pass.bind_pipeline(&self.grid_pipeline);
			let pc = DebugPushConstants {
				scene_address: scene_addr,
				data_address: 0,
			};
			pass.push_constant(rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel, 0, &pc);
			pass.draw(3, 1, 0, 0);
		}

		if line_count > 0
			&& let Some(buf) = &self.line_buffers[frame]
		{
			pass.bind_pipeline(&self.line_pipeline);
			let pc = DebugPushConstants {
				scene_address: scene_addr,
				data_address: buf.address(),
			};
			pass.push_constant(rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel, 0, &pc);
			pass.draw(line_count, 1, 0, 0);
		}

		if billboard_count > 0
			&& let Some(buf) = &self.billboard_buffers[frame]
		{
			pass.bind_pipeline(&self.billboard_pipeline);
			let pc = DebugPushConstants {
				scene_address: scene_addr,
				data_address: buf.address(),
			};
			pass.push_constant(rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel, 0, &pc);
			pass.draw(billboard_count * 6, 1, 0, 0);
		}
	}
}


fn safe_normalize(v: Vec3<f32>, fallback: Vec3<f32>) -> Vec3<f32> {
	let len_sq = v.x * v.x + v.y * v.y + v.z * v.z;
	if len_sq > 1e-8 {
		v / len_sq.sqrt()
	} else {
		fallback
	}
}

fn ortho_basis(dir: Vec3<f32>) -> (Vec3<f32>, Vec3<f32>) {
	let up = if dir.y.abs() < 0.99 {
		Vec3::new(0.0, 1.0, 0.0)
	} else {
		Vec3::new(1.0, 0.0, 0.0)
	};
	let u = safe_normalize(up.cross(&dir), Vec3::new(1.0, 0.0, 0.0));
	let v = safe_normalize(dir.cross(&u), Vec3::new(0.0, 1.0, 0.0));
	(u, v)
}

fn transform_point_proj(m: &Mat4x4<f32>, p: [f32; 4]) -> Vec3<f32> {
	let dot = |r: usize| {
		m.rows[r].x * p[0] + m.rows[r].y * p[1] + m.rows[r].z * p[2] + m.rows[r].w * p[3]
	};
	let x = dot(0);
	let y = dot(1);
	let z = dot(2);
	let w = dot(3);
	let inv_w = if w.abs() > 1e-6 { 1.0 / w } else { 1.0 };
	Vec3::new(x * inv_w, y * inv_w, z * inv_w)
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_debug_draw_primitives() {
		let mut dd = DebugDraw::new();
		dd.draw_grid = true;

		dd.wire_box(
			Vec3::ZERO,
			Vec3::new(0.5, 0.5, 0.5),
			Quat::IDENTITY,
			[0.0, 1.0, 0.0, 1.0],
		);
		assert_eq!(dd.lines.len(), 24, "wire_box has 12 edges = 24 vertices");

		dd.wire_frustum(Mat4x4::IDENTITY, [1.0, 0.0, 1.0, 1.0]);
		assert_eq!(dd.lines.len(), 48, "wire_frustum adds 24 vertices");

		dd.wire_sphere(Vec3::ZERO, 2.0, [1.0, 1.0, 0.0, 1.0]);
		assert_eq!(dd.lines.len(), 48 + 3 * 24 * 2);

		dd.wire_cone(
			Vec3::ZERO,
			Vec3::new(0.0, 0.0, -1.0),
			5.0,
			30.0,
			16,
			[1.0, 0.8, 0.2, 1.0],
		);
		assert_eq!(dd.lines.len() % 2, 0);

		dd.billboard(Vec3::new(0.0, 2.0, 0.0), 1.0, [1.0; 4], DebugIconKind::Camera);
		assert_eq!(dd.billboards.len(), 1);

		dd.clear();
		assert!(dd.is_empty());
		assert!(dd.draw_grid, "clear() must preserve draw_grid toggle");
	}
}
