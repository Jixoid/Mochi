/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::ffi::*;
use crate::{Device, Format, Shader, ShaderStage, VT};


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VertexInputRate {
	Vertex = 0,
	Instance = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DescriptorType {
	UniformBuffer = 6,
	UniformBufferDynamic = 8,
	StorageBuffer = 7,
	StorageBufferDynamic = 9,
	TextureSampler = 1,
	StorageImage = 3,
	SeparateImage = 2,
	SeparateSampler = 0,
	AccelerationStructure = 1000150000,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PipelineKind {
	Graphic = 0,
	Compute = 1,
	RayTracing = 1000165000,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PolygonMode {
	Fill = 0,
	Line = 1,
	Point = 2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PrimitiveTopology {
	PointList = 0,
	LineList = 1,
	LineStrip = 2,
	TriangleList = 3,
	TriangleStrip = 4,
	TriangleFan = 5,
	PatchList = 10,
}


// Helpers
pub struct PushConstantSlot {
	stage: ShaderStage,
	types: Vec<VT>,
}

impl PushConstantSlot {
	pub fn new(stage: ShaderStage, types: Vec<VT>) -> Self {
		Self { stage, types }
	}

	pub fn stage(&self) -> ShaderStage {
		self.stage
	}

	pub fn types(&self) -> &[VT] {
		&self.types
	}
}
pub type PushConstantList = Vec<PushConstantSlot>;

pub struct VertexBindSlot {
	stride: u64,
	items: Vec<VT>,
	input_rate: VertexInputRate,
	index: u8,
}

impl VertexBindSlot {
	pub fn new(stride: u64, items: Vec<VT>, input_rate: VertexInputRate, index: u8) -> Self {
		Self {
			stride,
			items,
			input_rate,
			index,
		}
	}

	pub fn items(&self) -> &[VT] {
		&self.items
	}

	pub fn stride(&self) -> u64 {
		self.stride
	}

	pub fn input_rate(&self) -> VertexInputRate {
		self.input_rate
	}

	pub fn index(&self) -> u8 {
		self.index
	}
}
pub type VertexBindList = Vec<VertexBindSlot>;


#[inline]
fn align_size(size: u64, alignment: u64) -> u64 {
	if alignment == 0 {
		size
	} else {
		((size + alignment - 1) / alignment) * alignment
	}
}


pub struct PipelineMetaInner {
	handle: MochiPipelineMetaHandle,
	push: PushConstantList,
	vert: VertexBindList,
}

unsafe impl Send for PipelineMetaInner {}
unsafe impl Sync for PipelineMetaInner {}

impl Drop for PipelineMetaInner {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_pipeline_meta_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}

#[derive(Clone)]
pub struct PipelineMeta {
	inner: std::sync::Arc<PipelineMetaInner>,
}

unsafe impl Send for PipelineMeta {}
unsafe impl Sync for PipelineMeta {}

impl PipelineMeta {
	pub fn new(push: PushConstantList, vert: VertexBindList) -> Self {
		let mut all_stages = ShaderStage::empty();
		let mut total_size: u64 = 0;
		let mut max_align: u64 = 0;

		for p in &push {
			all_stages |= p.stage();
			for typ in p.types() {
				max_align = max_align.max(typ.align());
				total_size = align_size(total_size, typ.align());
				total_size += typ.size() * typ.count() as u64;
			}
		}
		total_size = align_size(total_size, max_align);

		let handle = unsafe {
			mochi_rhi_pipeline_meta_new(
				all_stages.bits(),
				total_size as u32,
			)
		};
		assert!(!handle.is_null(), "RHI driver failed to create PipelineMeta");

		Self {
			inner: std::sync::Arc::new(PipelineMetaInner { handle, push, vert }),
		}
	}

	pub fn new_raw(push_stages: ShaderStage, push_size: u32) -> Self {
		let handle = unsafe {
			mochi_rhi_pipeline_meta_new(
				push_stages.bits(),
				push_size,
			)
		};
		assert!(!handle.is_null(), "RHI driver failed to create PipelineMeta");

		Self {
			inner: std::sync::Arc::new(PipelineMetaInner {
				handle,
				push: Vec::new(),
				vert: Vec::new(),
			}),
		}
	}

	pub fn inner(&self) -> &std::sync::Arc<PipelineMetaInner> {
		&self.inner
	}

	pub fn handle(&self) -> MochiPipelineMetaHandle {
		self.inner.handle
	}

	pub fn push(&self) -> &[PushConstantSlot] {
		&self.inner.push
	}

	pub fn vert(&self) -> &[VertexBindSlot] {
		&self.inner.vert
	}
}


pub struct PipelineInner {
	device: std::sync::Arc<crate::manager::DeviceInner>,
	meta: std::sync::Arc<PipelineMetaInner>,
	handle: MochiPipelineHandle,
	kind: PipelineKind,
}

unsafe impl Send for PipelineInner {}
unsafe impl Sync for PipelineInner {}

impl PipelineInner {
	pub fn device(&self) -> &std::sync::Arc<crate::manager::DeviceInner> {
		&self.device
	}

	pub fn meta(&self) -> &std::sync::Arc<PipelineMetaInner> {
		&self.meta
	}
}

impl Drop for PipelineInner {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_pipeline_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}

#[derive(Clone)]
pub struct Pipeline {
	inner: std::sync::Arc<PipelineInner>,
}

unsafe impl Send for Pipeline {}
unsafe impl Sync for Pipeline {}

impl Pipeline {
	pub fn new_graphics(
		device: &Device,
		meta: &PipelineMeta,
		shaders: &[&Shader],
		polymode: PolygonMode,
		primitive_topology: PrimitiveTopology,
		color_format: Format,
		depth_format: Format,
	) -> Self {
		let shader_handles: Vec<MochiShaderHandle> = shaders.iter().map(|s| s.handle()).collect();
		let handle = unsafe {
			mochi_rhi_pipeline_graphics_new(
				device.handle(),
				meta.handle(),
				shader_handles.as_ptr(),
				shader_handles.len() as u32,
				polymode as u8,
				primitive_topology as u8,
				color_format as u32,
				depth_format as u32,
			)
		};
		assert!(!handle.is_null(), "RHI driver failed to create Graphics Pipeline");

		Self {
			inner: std::sync::Arc::new(PipelineInner {
				device: device.inner().clone(),
				meta: meta.inner().clone(),
				handle,
				kind: PipelineKind::Graphic,
			}),
		}
	}

	pub fn inner(&self) -> &std::sync::Arc<PipelineInner> {
		&self.inner
	}

	pub fn device(&self) -> &std::sync::Arc<crate::manager::DeviceInner> {
		self.inner.device()
	}

	pub fn meta(&self) -> &std::sync::Arc<PipelineMetaInner> {
		self.inner.meta()
	}

	pub fn handle(&self) -> MochiPipelineHandle {
		self.inner.handle
	}

	pub fn kind(&self) -> PipelineKind {
		self.inner.kind
	}
}
