/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ffi::CString;
use std::sync::Arc;
use ash::vk;

use crate::shader::AshShader;
use crate::types::{
	to_vk_format, to_vk_polygon_mode, to_vk_primitive_topology, to_vk_shader_stage,
};
use mochi_rhi::error::{RhiError, RhiResult};
use mochi_rhi::format::Format;
use mochi_rhi::pipeline::{PipelineKind, PolygonMode, PrimitiveTopology};
use mochi_rhi::shader::ShaderStage;
use mochi_rhi::traits::{RhiPipeline, RhiPipelineMeta, RhiShader};


pub struct PipelineMetaInner {
	device: ash::Device,
	layout: vk::PipelineLayout,
}

unsafe impl Send for PipelineMetaInner {}
unsafe impl Sync for PipelineMetaInner {}

impl Drop for PipelineMetaInner {
	fn drop(&mut self) {
		if self.layout != vk::PipelineLayout::null() {
			unsafe {
				self.device.destroy_pipeline_layout(self.layout, None);
			}
			self.layout = vk::PipelineLayout::null();
		}
	}
}

#[derive(Clone)]
pub struct AshPipelineMeta {
	inner: Arc<PipelineMetaInner>,
}

unsafe impl Send for AshPipelineMeta {}
unsafe impl Sync for AshPipelineMeta {}

impl AshPipelineMeta {
	pub fn new(device: &ash::Device, layout: vk::PipelineLayout) -> Self {
		Self {
			inner: Arc::new(PipelineMetaInner {
				device: device.clone(),
				layout,
			}),
		}
	}

	pub fn create(
		device: &ash::Device,
		set_layout: Option<vk::DescriptorSetLayout>,
		push_stages: ShaderStage,
		push_size: u32,
	) -> RhiResult<Self> {
		let mut ranges = Vec::new();
		if push_size > 0 {
			ranges.push(vk::PushConstantRange {
				stage_flags: to_vk_shader_stage(push_stages),
				offset: 0,
				size: push_size,
			});
		}

		let mut set_layouts = Vec::new();
		if let Some(sl) = set_layout {
			set_layouts.push(sl);
		}

		let layout_info = vk::PipelineLayoutCreateInfo::default()
			.set_layouts(&set_layouts)
			.push_constant_ranges(&ranges);

		let layout = unsafe {
			device
				.create_pipeline_layout(&layout_info, None)
				.map_err(|e| RhiError::PipelineCreation(format!("Failed to create pipeline layout: {e}")))?
		};

		Ok(Self::new(device, layout))
	}

	pub fn layout(&self) -> vk::PipelineLayout {
		self.inner.layout
	}
}

impl RhiPipelineMeta for AshPipelineMeta {}


pub struct PipelineInner {
	device: ash::Device,
	handle: vk::Pipeline,
	layout: vk::PipelineLayout,
	descriptor_set: Option<vk::DescriptorSet>,
	kind: PipelineKind,
}

unsafe impl Send for PipelineInner {}
unsafe impl Sync for PipelineInner {}

impl Drop for PipelineInner {
	fn drop(&mut self) {
		if self.handle != vk::Pipeline::null() {
			unsafe {
				self.device.destroy_pipeline(self.handle, None);
			}
			self.handle = vk::Pipeline::null();
		}
	}
}

#[derive(Clone)]
pub struct AshPipeline {
	inner: Arc<PipelineInner>,
}

unsafe impl Send for AshPipeline {}
unsafe impl Sync for AshPipeline {}

impl AshPipeline {
	pub fn new(
		device: &ash::Device,
		handle: vk::Pipeline,
		layout: vk::PipelineLayout,
		descriptor_set: Option<vk::DescriptorSet>,
		kind: PipelineKind,
	) -> Self {
		Self {
			inner: Arc::new(PipelineInner {
				device: device.clone(),
				handle,
				layout,
				descriptor_set,
				kind,
			}),
		}
	}

	pub fn create_graphics(
		device: &ash::Device,
		meta: &AshPipelineMeta,
		shaders: &[&AshShader],
		polymode: PolygonMode,
		primitive_topology: PrimitiveTopology,
		color_format: Format,
		depth_format: Format,
		descriptor_set: Option<vk::DescriptorSet>,
	) -> RhiResult<Self> {
		let mut c_entries = Vec::with_capacity(shaders.len());

		for sh in shaders {
			let c_entry = CString::new(sh.entry())
				.map_err(|e| RhiError::PipelineCreation(format!("Invalid shader entry name: {e}")))?;
			c_entries.push(c_entry);
		}

		let mut shader_stages = Vec::with_capacity(shaders.len());
		for (i, sh) in shaders.iter().enumerate() {
			let stage_info = vk::PipelineShaderStageCreateInfo::default()
				.stage(to_vk_shader_stage(sh.stage()))
				.module(sh.handle())
				.name(&c_entries[i]);
			shader_stages.push(stage_info);
		}

		let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
		let dynamic_state = vk::PipelineDynamicStateCreateInfo::default()
			.dynamic_states(&dynamic_states);

		let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();

		let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
			.topology(to_vk_primitive_topology(primitive_topology))
			.primitive_restart_enable(false);

		let viewport_state = vk::PipelineViewportStateCreateInfo::default()
			.viewport_count(1)
			.scissor_count(1);

		let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
			.depth_clamp_enable(false)
			.rasterizer_discard_enable(false)
			.polygon_mode(to_vk_polygon_mode(polymode))
			.cull_mode(vk::CullModeFlags::BACK)
			.front_face(vk::FrontFace::COUNTER_CLOCKWISE)
			.depth_bias_enable(false)
			.line_width(1.0);

		let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
			.sample_shading_enable(false)
			.rasterization_samples(vk::SampleCountFlags::TYPE_1);

		let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
			.depth_test_enable(true)
			.depth_write_enable(true)
			.depth_compare_op(vk::CompareOp::LESS)
			.depth_bounds_test_enable(false)
			.stencil_test_enable(false);

		let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
			.color_write_mask(vk::ColorComponentFlags::RGBA)
			.blend_enable(true)
			.src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
			.dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
			.color_blend_op(vk::BlendOp::ADD)
			.src_alpha_blend_factor(vk::BlendFactor::ONE)
			.dst_alpha_blend_factor(vk::BlendFactor::ZERO)
			.alpha_blend_op(vk::BlendOp::ADD);

		let attachments = [color_blend_attachment];
		let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
			.logic_op_enable(false)
			.attachments(&attachments);

		let color_formats = [to_vk_format(color_format)];
		let vk_depth_fmt = to_vk_format(depth_format);

		let mut rendering_info = vk::PipelineRenderingCreateInfo::default()
			.color_attachment_formats(&color_formats)
			.depth_attachment_format(vk_depth_fmt);

		let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
			.stages(&shader_stages)
			.vertex_input_state(&vertex_input)
			.input_assembly_state(&input_assembly)
			.viewport_state(&viewport_state)
			.rasterization_state(&rasterizer)
			.multisample_state(&multisampling)
			.depth_stencil_state(&depth_stencil)
			.color_blend_state(&color_blending)
			.dynamic_state(&dynamic_state)
			.layout(meta.layout())
			.push_next(&mut rendering_info);

		let pipelines = unsafe {
			device
				.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
				.map_err(|(_, e)| RhiError::PipelineCreation(format!("Failed to create graphics pipeline: {e}")))?
		};

		Ok(Self::new(device, pipelines[0], meta.layout(), descriptor_set, PipelineKind::Graphic))
	}

	pub fn handle(&self) -> vk::Pipeline {
		self.inner.handle
	}

	pub fn layout(&self) -> vk::PipelineLayout {
		self.inner.layout
	}

	pub fn descriptor_set(&self) -> Option<vk::DescriptorSet> {
		self.inner.descriptor_set
	}

	pub fn kind(&self) -> PipelineKind {
		self.inner.kind
	}
}

impl RhiPipeline for AshPipeline {
	fn kind(&self) -> PipelineKind {
		self.inner.kind
	}
}
