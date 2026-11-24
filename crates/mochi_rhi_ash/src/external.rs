/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


#[cfg(unix)]
use std::os::unix::io::RawFd;

use ash::vk;
use mochi_rhi::error::{RhiError, RhiResult};
use mochi_rhi::format::Format;

use crate::device::AshDevice;
use crate::swapchain::AshRenderTarget;
use crate::types::to_vk_format;


pub const EXTERNAL_SHARED_IMAGE_USAGE: vk::ImageUsageFlags = vk::ImageUsageFlags::from_raw(
	vk::ImageUsageFlags::COLOR_ATTACHMENT.as_raw()
		| vk::ImageUsageFlags::SAMPLED.as_raw()
		| vk::ImageUsageFlags::TRANSFER_DST.as_raw(),
);


pub struct ExportableImage {
	device: ash::Device,
	image: vk::Image,
	memory: vk::DeviceMemory,
	view: vk::ImageView,
	allocation_size: u64,
	width: u32,
	height: u32,
	format: Format,
}

unsafe impl Send for ExportableImage {}
unsafe impl Sync for ExportableImage {}

impl ExportableImage {
	#[cfg(unix)]
	pub fn new(ash_device: &AshDevice, width: u32, height: u32, format: Format) -> RhiResult<Self> {
		let vk_dev = ash_device.device();
		let vk_format = to_vk_format(format);
		let width = width.max(1);
		let height = height.max(1);

		let mut ext_image_info = vk::ExternalMemoryImageCreateInfo::default()
			.handle_types(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);

		let image_info = vk::ImageCreateInfo::default()
			.image_type(vk::ImageType::TYPE_2D)
			.format(vk_format)
			.extent(vk::Extent3D {
				width,
				height,
				depth: 1,
			})
			.mip_levels(1)
			.array_layers(1)
			.samples(vk::SampleCountFlags::TYPE_1)
			.tiling(vk::ImageTiling::OPTIMAL)
			.usage(EXTERNAL_SHARED_IMAGE_USAGE)
			.sharing_mode(vk::SharingMode::EXCLUSIVE)
			.initial_layout(vk::ImageLayout::UNDEFINED)
			.push_next(&mut ext_image_info);

		let image = unsafe {
			vk_dev
				.create_image(&image_info, None)
				.map_err(|e| RhiError::ResourceCreation(format!("Failed to create exportable image: {e}")))?
		};

		let mem_reqs = unsafe { vk_dev.get_image_memory_requirements(image) };
		let mem_props = unsafe {
			ash_device
				.instance()
				.get_physical_device_memory_properties(ash_device.physical_device())
		};

		let mem_type_index = find_memory_type(
			&mem_props,
			mem_reqs.memory_type_bits,
			vk::MemoryPropertyFlags::DEVICE_LOCAL,
		)
		.ok_or_else(|| {
			unsafe { vk_dev.destroy_image(image, None) };
			RhiError::ResourceCreation(
				"No compatible DEVICE_LOCAL memory type for exportable image".into(),
			)
		})?;

		let mut export_info = vk::ExportMemoryAllocateInfo::default()
			.handle_types(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);
		let mut dedicated_info = vk::MemoryDedicatedAllocateInfo::default().image(image);

		let alloc_info = vk::MemoryAllocateInfo::default()
			.allocation_size(mem_reqs.size)
			.memory_type_index(mem_type_index)
			.push_next(&mut export_info)
			.push_next(&mut dedicated_info);

		let memory = unsafe {
			vk_dev.allocate_memory(&alloc_info, None).map_err(|e| {
				vk_dev.destroy_image(image, None);
				RhiError::ResourceCreation(format!("Failed to allocate exportable memory: {e}"))
			})?
		};

		if let Err(e) = unsafe { vk_dev.bind_image_memory(image, memory, 0) } {
			unsafe {
				vk_dev.destroy_image(image, None);
				vk_dev.free_memory(memory, None);
			}
			return Err(RhiError::ResourceCreation(format!(
				"Failed to bind exportable image memory: {e}"
			)));
		}

		let view_info = vk::ImageViewCreateInfo::default()
			.image(image)
			.view_type(vk::ImageViewType::TYPE_2D)
			.format(vk_format)
			.subresource_range(vk::ImageSubresourceRange {
				aspect_mask: vk::ImageAspectFlags::COLOR,
				base_mip_level: 0,
				level_count: 1,
				base_array_layer: 0,
				layer_count: 1,
			});

		let view = unsafe {
			vk_dev.create_image_view(&view_info, None).map_err(|e| {
				vk_dev.destroy_image(image, None);
				vk_dev.free_memory(memory, None);
				RhiError::ResourceCreation(format!("Failed to create exportable image view: {e}"))
			})?
		};

		Ok(Self {
			device: vk_dev.clone(),
			image,
			memory,
			view,
			allocation_size: mem_reqs.size,
			width,
			height,
			format,
		})
	}

	#[cfg(unix)]
	pub fn export_fd(&self, ash_device: &AshDevice) -> RhiResult<RawFd> {
		let ext_fd = ash::khr::external_memory_fd::Device::new(
			ash_device.instance(),
			ash_device.device(),
		);
		let get_fd_info = vk::MemoryGetFdInfoKHR::default()
			.memory(self.memory)
			.handle_type(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);

		unsafe {
			ext_fd
				.get_memory_fd(&get_fd_info)
				.map_err(|e| RhiError::BackendError(format!("Failed to export memory fd: {e}")))
		}
	}

	pub fn as_render_target(
		&self,
		depth_image: vk::Image,
		depth_view: vk::ImageView,
		depth_format: Format,
	) -> AshRenderTarget {
		AshRenderTarget {
			color_image: self.image,
			color_view: self.view,
			resolve_image: vk::Image::null(),
			resolve_view: vk::ImageView::null(),
			color_format: self.format,
			depth_image,
			depth_view,
			resolve_depth_image: vk::Image::null(),
			resolve_depth_view: vk::ImageView::null(),
			depth_format,
			extent: vk::Extent2D {
				width: self.width,
				height: self.height,
			},
			final_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
			samples: mochi_rhi::MsaaSamples::Off,
		}
	}

	pub fn allocation_size(&self) -> u64 {
		self.allocation_size
	}

	pub fn width(&self) -> u32 {
		self.width
	}

	pub fn height(&self) -> u32 {
		self.height
	}

	pub fn format(&self) -> Format {
		self.format
	}

	pub fn image(&self) -> vk::Image {
		self.image
	}

	pub fn view(&self) -> vk::ImageView {
		self.view
	}
}

impl Drop for ExportableImage {
	fn drop(&mut self) {
		unsafe {
			let _ = self.device.device_wait_idle();
			self.device.destroy_image_view(self.view, None);
			self.device.destroy_image(self.image, None);
			self.device.free_memory(self.memory, None);
		}
	}
}


fn find_memory_type(
	mem_props: &vk::PhysicalDeviceMemoryProperties,
	type_filter: u32,
	required: vk::MemoryPropertyFlags,
) -> Option<u32> {
	(0..mem_props.memory_type_count).find(|&i| {
		(type_filter & (1 << i)) != 0
			&& mem_props.memory_types[i as usize]
				.property_flags
				.contains(required)
	})
}
