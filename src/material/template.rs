/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use crate::material::types::{BlendMode, PassKind};
use crate::rhi::{self, Format, PipelineMeta, PolygonMode, PrimitiveTopology, Shader, ShaderStage};


static NEXT_TEMPLATE_ID: AtomicU16 = AtomicU16::new(1);

pub struct MaterialTemplate {
	id: u16,
	blend_mode: BlendMode,
	forward_pipeline: RwLock<rhi::Pipeline>,
	shadow_pipeline: Option<rhi::Pipeline>,
	depth_pipeline: Option<rhi::Pipeline>,
	meta: rhi::PipelineMeta,
	vert_shader: rhi::Shader,
	frag_shader: rhi::Shader,
	color_format: Format,
	depth_format: Format,
	samples: AtomicU32,
}

impl MaterialTemplate {
	pub fn new_forward(
		device: &rhi::Device,
		vert_spv: &[u8],
		frag_spv: &[u8],
		blend_mode: BlendMode,
		color_format: Format,
		depth_format: Format,
		push_constant_size: u32,
		samples: rhi::MsaaSamples,
	) -> Arc<Self> {
		let id = NEXT_TEMPLATE_ID.fetch_add(1, Ordering::Relaxed);

		let vert = Shader::new(device, ShaderStage::Vertex, vert_spv, "main");
		let frag = Shader::new(device, ShaderStage::Pixel, frag_spv, "main");

		let meta = PipelineMeta::new_raw(
			ShaderStage::Vertex | ShaderStage::Pixel,
			push_constant_size,
		);

		let (blend_enable, depth_write) = match blend_mode {
			BlendMode::Opaque | BlendMode::Masked { .. } => (false, true),
			BlendMode::Transparent | BlendMode::Additive => (true, false),
		};

		let forward_pipeline = rhi::Pipeline::new_graphics(
			device,
			&meta,
			&[&vert, &frag],
			PolygonMode::Fill,
			PrimitiveTopology::TriangleList,
			color_format,
			depth_format,
			blend_enable,
			depth_write,
			samples,
		);

		Arc::new(Self {
			id,
			blend_mode,
			forward_pipeline: RwLock::new(forward_pipeline),
			shadow_pipeline: None,
			depth_pipeline: None,
			meta,
			vert_shader: vert,
			frag_shader: frag,
			color_format,
			depth_format,
			samples: AtomicU32::new(samples.count()),
		})
	}

	pub fn id(&self) -> u16 {
		self.id
	}

	pub fn blend_mode(&self) -> BlendMode {
		self.blend_mode
	}

	pub fn set_msaa(&self, device: &rhi::Device, samples: rhi::MsaaSamples) {
		let (blend_enable, depth_write) = match self.blend_mode {
			BlendMode::Opaque | BlendMode::Masked { .. } => (false, true),
			BlendMode::Transparent | BlendMode::Additive => (true, false),
		};

		let new_pipe = rhi::Pipeline::new_graphics(
			device,
			&self.meta,
			&[&self.vert_shader, &self.frag_shader],
			PolygonMode::Fill,
			PrimitiveTopology::TriangleList,
			self.color_format,
			self.depth_format,
			blend_enable,
			depth_write,
			samples,
		);

		*self.forward_pipeline.write().unwrap() = new_pipe;
		self.samples.store(samples.count(), Ordering::Relaxed);
	}

	pub fn pipeline(&self, pass: PassKind) -> Option<rhi::Pipeline> {
		match pass {
			PassKind::ForwardLit => Some(self.forward_pipeline.read().unwrap().clone()),
			PassKind::ShadowCaster => self.shadow_pipeline.clone().or_else(|| Some(self.forward_pipeline.read().unwrap().clone())),
			PassKind::DepthPrepass => self.depth_pipeline.clone().or_else(|| Some(self.forward_pipeline.read().unwrap().clone())),
		}
	}
}
