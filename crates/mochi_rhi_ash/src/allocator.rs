/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};
use ash::vk;
use gpu_allocator::vulkan::{AllocationCreateDesc, AllocationScheme, Allocator};
use mochi_math::Extent;

use crate::buffer::AshBuffer;
use crate::image::AshImage;
use crate::sampler::AshSampler;
use crate::types::{
	to_vk_address_mode, to_vk_buffer_usage, to_vk_filter, to_vk_format, to_vk_image_tiling, to_vk_image_usage,
};
use crate::AshVulkanBackend;
use mochi_rhi::error::{RhiError, RhiResult};
use mochi_rhi::format::Format;
use mochi_rhi::manager::{AllocationCreate, AllocationLocation};
use mochi_rhi::sampler::{SamplerAddressMode, SamplerFilter};
use mochi_rhi::traits::RhiAllocator;
use mochi_rhi::{BufferUsage, ImageTiling, ImageUsage};


#[derive(Clone)]
pub struct AshAllocator {
	device: ash::Device,
	allocator: Arc<Mutex<Allocator>>,
}

unsafe impl Send for AshAllocator {}
unsafe impl Sync for AshAllocator {}

impl AshAllocator {
	pub fn new(device: &ash::Device, allocator: Arc<Mutex<Allocator>>) -> Self {
		Self {
			device: device.clone(),
			allocator,
		}
	}

	pub fn allocator(&self) -> &Arc<Mutex<Allocator>> {
		&self.allocator
	}
}

impl RhiAllocator<AshVulkanBackend> for AshAllocator {
	fn alloc_buffer(
		&self,
		size: u64,
		usage: BufferUsage,
		create: AllocationCreate,
		location: AllocationLocation,
	) -> RhiResult<AshBuffer> {
		let vk_usage = to_vk_buffer_usage(usage);
		let buffer_info = vk::BufferCreateInfo::default()
			.size(size)
			.usage(vk_usage)
			.sharing_mode(vk::SharingMode::EXCLUSIVE);

		let buffer = unsafe {
			self.device
				.create_buffer(&buffer_info, None)
				.map_err(|e| RhiError::ResourceCreation(format!("Failed to create buffer: {e}")))?
		};

		let requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };

		let mem_location = match location {
			AllocationLocation::PreferHost => gpu_allocator::MemoryLocation::CpuToGpu,
			AllocationLocation::PreferDevice => gpu_allocator::MemoryLocation::GpuOnly,
			AllocationLocation::Auto => {
				if create.contains(AllocationCreate::HostSequentialWrite)
					|| create.contains(AllocationCreate::HostRandomAccess)
					|| create.contains(AllocationCreate::Mapped)
				{
					gpu_allocator::MemoryLocation::CpuToGpu
				} else {
					gpu_allocator::MemoryLocation::GpuOnly
				}
			}
		};

		let alloc_desc = AllocationCreateDesc {
			name: "AshBuffer",
			requirements,
			location: mem_location,
			linear: true,
			allocation_scheme: AllocationScheme::GpuAllocatorManaged,
		};

		let allocation = self
			.allocator
			.lock()
			.unwrap()
			.allocate(&alloc_desc)
			.map_err(|_e| RhiError::OutOfMemory)?;

		unsafe {
			self.device
				.bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
				.map_err(|e| RhiError::BackendError(format!("Failed to bind buffer memory: {e}")))?;
		}

		let device_address = if usage.contains(BufferUsage::DeviceAddress) {
			let addr_info = vk::BufferDeviceAddressInfo::default().buffer(buffer);
			unsafe { self.device.get_buffer_device_address(&addr_info) }
		} else {
			0
		};

		Ok(AshBuffer::new(
			&self.device,
			self.allocator.clone(),
			buffer,
			allocation,
			size,
			device_address,
		))
	}

	fn alloc_image(
		&self,
		ext: Extent<2, u32>,
		format: Format,
		usage: ImageUsage,
		tiling: ImageTiling,
		_create: AllocationCreate,
		location: AllocationLocation,
	) -> RhiResult<AshImage> {
		let vk_format = to_vk_format(format);
		let vk_usage = to_vk_image_usage(usage);
		let vk_tiling = to_vk_image_tiling(tiling);

		let extent = vk::Extent3D {
			width: ext[0],
			height: ext[1],
			depth: 1,
		};

		let image_info = vk::ImageCreateInfo::default()
			.image_type(vk::ImageType::TYPE_2D)
			.format(vk_format)
			.extent(extent)
			.mip_levels(1)
			.array_layers(1)
			.samples(vk::SampleCountFlags::TYPE_1)
			.tiling(vk_tiling)
			.usage(vk_usage)
			.sharing_mode(vk::SharingMode::EXCLUSIVE)
			.initial_layout(vk::ImageLayout::UNDEFINED);

		let image = unsafe {
			self.device
				.create_image(&image_info, None)
				.map_err(|e| RhiError::ResourceCreation(format!("Failed to create image: {e}")))?
		};

		let requirements = unsafe { self.device.get_image_memory_requirements(image) };

		let mem_location = match location {
			AllocationLocation::PreferHost => gpu_allocator::MemoryLocation::CpuToGpu,
			_ => gpu_allocator::MemoryLocation::GpuOnly,
		};

		let alloc_desc = AllocationCreateDesc {
			name: "AshImage",
			requirements,
			location: mem_location,
			linear: tiling == ImageTiling::Linear,
			allocation_scheme: AllocationScheme::GpuAllocatorManaged,
		};

		let allocation = self
			.allocator
			.lock()
			.unwrap()
			.allocate(&alloc_desc)
			.map_err(|_e| RhiError::OutOfMemory)?;

		unsafe {
			self.device
				.bind_image_memory(image, allocation.memory(), allocation.offset())
				.map_err(|e| RhiError::BackendError(format!("Failed to bind image memory: {e}")))?;
		}

		Ok(AshImage::new(
			&self.device,
			self.allocator.clone(),
			image,
			allocation,
			ext,
			format,
		))
	}

	fn alloc_sampler(
		&self,
		address_mode: Extent<3, SamplerAddressMode>,
		mag_filter: SamplerFilter,
		min_filter: SamplerFilter,
	) -> RhiResult<AshSampler> {
		let sampler_info = vk::SamplerCreateInfo::default()
			.mag_filter(to_vk_filter(mag_filter))
			.min_filter(to_vk_filter(min_filter))
			.address_mode_u(to_vk_address_mode(address_mode[0]))
			.address_mode_v(to_vk_address_mode(address_mode[1]))
			.address_mode_w(to_vk_address_mode(address_mode[2]))
			.anisotropy_enable(true)
			.max_anisotropy(16.0)
			.border_color(vk::BorderColor::INT_OPAQUE_BLACK)
			.unnormalized_coordinates(false)
			.compare_enable(false)
			.compare_op(vk::CompareOp::ALWAYS)
			.mipmap_mode(vk::SamplerMipmapMode::LINEAR);

		let sampler = unsafe {
			self.device
				.create_sampler(&sampler_info, None)
				.map_err(|e| RhiError::ResourceCreation(format!("Failed to create sampler: {e}")))?
		};

		Ok(AshSampler::new(&self.device, sampler))
	}
}
