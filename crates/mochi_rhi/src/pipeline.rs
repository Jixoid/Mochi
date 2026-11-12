/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use bitflags::bitflags;

use crate::format::{Format, VT};
use crate::manager::device::Device;
use crate::shader::{Shader, ShaderStage};
use crate::traits::{RhiBackend, RhiDevice};
use crate::types::MsaaSamples;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DescriptorType {
	Sampler = 0,
	CombinedImageSampler = 1,
	SampledImage = 2,
	StorageImage = 3,
	UniformTexelBuffer = 4,
	StorageTexelBuffer = 5,
	UniformBuffer = 6,
	StorageBuffer = 7,
	UniformBufferDynamic = 8,
	StorageBufferDynamic = 9,
	InputAttachment = 10,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PipelineKind {
	Graphic,
	Compute,
	RayTracing,
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
	PatchList = 9,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VertexInputRate {
	Vertex = 0,
	Instance = 1,
}

bitflags! {
	#[derive(Copy, Clone, Debug, PartialEq, Eq)]
	pub struct DynamicState: u32 {
		const Viewport = 0x00000001;
		const Scissor  = 0x00000002;
	}
}


#[derive(Clone, Debug, PartialEq, Eq)]
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


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VertexBindSlot {
	items: Vec<VT>,
	stride: u64,
	input_rate: VertexInputRate,
	index: u8,
}

impl VertexBindSlot {
	pub fn new(items: Vec<VT>, stride: u64, input_rate: VertexInputRate, index: u8) -> Self {
		Self { items, stride, input_rate, index }
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


#[derive(Clone)]
pub struct PipelineMeta<B: RhiBackend> {
	push_stages: ShaderStage,
	push_size: u32,
	push: PushConstantList,
	vert: VertexBindList,
	raw: Arc<Mutex<Option<Arc<B::PipelineMeta>>>>,
	_marker: PhantomData<B>,
}

unsafe impl<B: RhiBackend> Send for PipelineMeta<B> {}
unsafe impl<B: RhiBackend> Sync for PipelineMeta<B> {}

impl<B: RhiBackend> PipelineMeta<B> {
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

		Self {
			push_stages: all_stages,
			push_size: total_size as u32,
			push,
			vert,
			raw: Arc::new(Mutex::new(None)),
			_marker: PhantomData,
		}
	}

	pub fn new_raw(push_stages: ShaderStage, push_size: u32) -> Self {
		Self {
			push_stages,
			push_size,
			push: Vec::new(),
			vert: Vec::new(),
			raw: Arc::new(Mutex::new(None)),
			_marker: PhantomData,
		}
	}

	pub fn push_stages(&self) -> ShaderStage {
		self.push_stages
	}

	pub fn push_size(&self) -> u32 {
		self.push_size
	}

	pub fn push(&self) -> &[PushConstantSlot] {
		&self.push
	}

	pub fn vert(&self) -> &[VertexBindSlot] {
		&self.vert
	}

	pub fn get_or_create_raw(&self, device: &Device<B>) -> Arc<B::PipelineMeta> {
		let mut lock = self.raw.lock().unwrap();
		if let Some(existing) = &*lock {
			return existing.clone();
		}
		let meta = device
			.raw()
			.create_pipeline_meta(self.push_stages, self.push_size)
			.expect("Failed to create pipeline meta");
		let arc_meta = Arc::new(meta);
		*lock = Some(arc_meta.clone());
		arc_meta
	}
}


#[derive(Clone)]
pub struct Pipeline<B: RhiBackend> {
	inner: Arc<B::Pipeline>,
	kind: PipelineKind,
}

unsafe impl<B: RhiBackend> Send for Pipeline<B> {}
unsafe impl<B: RhiBackend> Sync for Pipeline<B> {}

impl<B: RhiBackend> Pipeline<B> {
	pub fn new_graphics(
		device: &Device<B>,
		meta: &PipelineMeta<B>,
		shaders: &[&Shader<B>],
		polymode: PolygonMode,
		primitive_topology: PrimitiveTopology,
		color_format: Format,
		depth_format: Format,
		blend_enable: bool,
		depth_write: bool,
		samples: MsaaSamples,
	) -> Self {
		let raw_meta = meta.get_or_create_raw(device);
		let raw_shaders: Vec<&B::Shader> = shaders.iter().map(|s| s.raw()).collect();

		let pipe = device
			.raw()
			.create_graphics_pipeline(
				&*raw_meta,
				&raw_shaders,
				polymode,
				primitive_topology,
				color_format,
				depth_format,
				blend_enable,
				depth_write,
				samples,
			)
			.expect("Failed to create Graphics Pipeline");

		Self {
			inner: Arc::new(pipe),
			kind: PipelineKind::Graphic,
		}
	}

	pub fn new_compute(
		device: &Device<B>,
		meta: &PipelineMeta<B>,
		shader: &Shader<B>,
	) -> Self {
		let raw_meta = meta.get_or_create_raw(device);
		let pipe = device
			.raw()
			.create_compute_pipeline(&*raw_meta, shader.raw())
			.expect("Failed to create Compute Pipeline");

		Self {
			inner: Arc::new(pipe),
			kind: PipelineKind::Compute,
		}
	}

	pub fn raw(&self) -> &B::Pipeline {
		&self.inner
	}

	pub fn kind(&self) -> PipelineKind {
		self.kind
	}
}
