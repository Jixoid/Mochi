/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::rhi;


static FULLSCREEN_VERT_SPV: &[u8] = include_bytes!("../../res/shaders/fullscreen.vert.spv");
static VELOCITY_FRAG_SPV: &[u8] = include_bytes!("../../res/shaders/velocity.frag.spv");


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VelocityPushConstants {
	pub scene_addr: u64,
	pub depth_tex_id: u32,
	pub _pad: u32,
}

pub struct VelocityTarget {
	pub width: u32,
	pub height: u32,
	pub image: rhi::Image2,
	pub view: rhi::ImageView2,
	pub descriptor_id: u32,
}

pub struct VelocityManager {
	pub pipeline: rhi::Pipeline,
	pub meta: rhi::PipelineMeta,
	pub sampler: rhi::Sampler2,
	pub target: Option<VelocityTarget>,
}

impl VelocityManager {
	pub fn new(device: &rhi::Device) -> Self {
		let vert = rhi::Shader::new(
			device,
			rhi::ShaderStage::Vertex,
			FULLSCREEN_VERT_SPV,
			"main",
		);
		let frag = rhi::Shader::new(
			device,
			rhi::ShaderStage::Pixel,
			VELOCITY_FRAG_SPV,
			"main",
		);
		let meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Pixel,
			std::mem::size_of::<VelocityPushConstants>() as u32,
		);
		let pipeline = rhi::Pipeline::new_graphics(
			device,
			&meta,
			&[&vert, &frag],
			rhi::PolygonMode::Fill,
			rhi::PrimitiveTopology::TriangleList,
			rhi::Format::V2Float16,
			rhi::Format::Undefined,
			false,
			false,
			rhi::MsaaSamples::Off,
		);

		let sampler = device
			.create_sampler(
				rhi::SamplerFilter::Nearest,
				rhi::SamplerAddressMode::ClampToEdge,
			)
			.expect("Failed to create velocity nearest sampler");

		Self {
			pipeline,
			meta,
			sampler,
			target: None,
		}
	}

	pub fn ensure_target(
		&mut self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		width: u32,
		height: u32,
	) {
		if let Some(target) = &self.target {
			if target.width == width && target.height == height {
				return;
			}
		}

		let ext = mochi_math::Extent::new([width, height]);
		let image = rhi::Image2::new(
			alloc,
			ext,
			rhi::Format::V2Float16,
			rhi::ImageUsage::ColorAttachment | rhi::ImageUsage::Sampled,
			rhi::ImageTiling::Optimal,
			rhi::AllocationCreate::Dedicated,
			rhi::AllocationLocation::PreferDevice,
		);
		let view = device
			.create_image_view(&image, rhi::Format::V2Float16)
			.expect("Failed to create velocity image view");
		let descriptor_id = device.write_texture_descriptor(&view, &self.sampler);

		self.target = Some(VelocityTarget {
			width,
			height,
			image,
			view,
			descriptor_id,
		});
	}

	pub fn free_target(&mut self) {
		self.target = None;
	}
}
