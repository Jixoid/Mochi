/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::rhi;
use mochi_rhi::Format;


pub struct Texture {
	pub image: rhi::Image2,
	pub view: rhi::ImageView2,
	pub sampler: rhi::Sampler2,
	pub descriptor_id: u32,
	pub width: u32,
	pub height: u32,
	pub format: Format,
}


pub struct TextureManager {
	textures: Vec<Texture>,
	default_sampler: Option<rhi::Sampler2>,
}

impl TextureManager {
	pub fn new(device: &rhi::Device, alloc: &rhi::Allocator, cmdmng: &rhi::CommandManager) -> Self {
		let sampler = device
			.create_sampler(
				mochi_rhi::sampler::SamplerFilter::Linear,
				mochi_rhi::sampler::SamplerAddressMode::Repeat,
			)
			.expect("Failed to create default texture sampler");

		let mut manager = Self {
			textures: Vec::new(),
			default_sampler: Some(sampler),
		};

		// Slot 0: Fallback White (Albedo / AO)
		let white_id = manager.load_raw_rgba8(
			device,
			alloc,
			cmdmng,
			1,
			1,
			Format::RGBA8_SRGB,
			&[255, 255, 255, 255],
		);

		// Slot 1: Fallback Normal (+Z flat normal map)
		let normal_id = manager.load_raw_rgba8(
			device,
			alloc,
			cmdmng,
			1,
			1,
			Format::RGBA8_UNORM,
			&[128, 128, 255, 255],
		);

		// Slot 2: Fallback Neutral ORM (Occlusion=1.0, Roughness=1.0, Metallic=0.0)
		let orm_id = manager.load_raw_rgba8(
			device,
			alloc,
			cmdmng,
			1,
			1,
			Format::RGBA8_UNORM,
			&[255, 255, 0, 255],
		);

		// Slot 3: Fallback Black (Emissive=0.0)
		let emissive_id = manager.load_raw_rgba8(
			device,
			alloc,
			cmdmng,
			1,
			1,
			Format::RGBA8_SRGB,
			&[0, 0, 0, 255],
		);

		crate::mochi_disp!(
			"TextureManager initialized with fallback slots: White={white_id}, Normal={normal_id}, ORM={orm_id}, Emissive={emissive_id}"
		);

		manager
	}

	pub fn load_raw_rgba8(
		&mut self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		cmdmng: &rhi::CommandManager,
		width: u32,
		height: u32,
		format: Format,
		data: &[u8],
	) -> u32 {
		let ext = mochi_math::Extent::new([width, height]);
		let image = rhi::Image2::new(
			alloc,
			ext,
			format,
			mochi_rhi::ImageUsage::Sampled | mochi_rhi::ImageUsage::TransferDst,
			mochi_rhi::ImageTiling::Optimal,
			mochi_rhi::manager::AllocationCreate::Dedicated,
			mochi_rhi::manager::AllocationLocation::PreferDevice,
		);

		device
			.upload_image_data(alloc, cmdmng.raw(), &image, width, height, data)
			.expect("Failed to upload image data to GPU");

		let view = device
			.create_image_view(&image, format)
			.expect("Failed to create image view");

		let sampler = self.default_sampler.as_ref().unwrap().clone();
		let descriptor_id = device.write_texture_descriptor(&view, &sampler);

		self.textures.push(Texture {
			image,
			view,
			sampler,
			descriptor_id,
			width,
			height,
			format,
		});

		descriptor_id
	}

	pub fn default_albedo_id(&self) -> u32 {
		0
	}

	pub fn default_normal_id(&self) -> u32 {
		1
	}

	pub fn default_orm_id(&self) -> u32 {
		2
	}

	pub fn default_emissive_id(&self) -> u32 {
		3
	}

	pub fn texture_count(&self) -> usize {
		self.textures.len()
	}
}
