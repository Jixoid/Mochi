/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};
use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};

use crate::sync::AshSemaphore;
use crate::types::from_vk_format;
use crate::AshVulkanBackend;
use mochi_rhi::error::{RhiError, RhiResult};
use mochi_rhi::format::Format;
use mochi_rhi::traits::{RhiRenderTarget, RhiSwapchain};


#[derive(Clone, Copy)]
pub struct AshRenderTarget {
	pub color_image: vk::Image,
	pub color_view: vk::ImageView,
	pub color_format: Format,
	pub depth_image: vk::Image,
	pub depth_view: vk::ImageView,
	pub depth_format: Format,
	pub extent: vk::Extent2D,
	pub final_layout: vk::ImageLayout,
}

unsafe impl Send for AshRenderTarget {}
unsafe impl Sync for AshRenderTarget {}

impl RhiRenderTarget for AshRenderTarget {
	fn color_format(&self) -> Format {
		self.color_format
	}

	fn depth_format(&self) -> Format {
		self.depth_format
	}
}


pub struct SwapchainInner {
	device: ash::Device,
	swapchain_loader: ash::khr::swapchain::Device,
	surface_loader: ash::khr::surface::Instance,
	physical_device: vk::PhysicalDevice,
	allocator: Arc<Mutex<Allocator>>,
	queue: vk::Queue,
	surface: vk::SurfaceKHR,
	swapchain: vk::SwapchainKHR,
	color_format: vk::Format,
	depth_format: vk::Format,
	extent: vk::Extent2D,
	images: Vec<vk::Image>,
	image_views: Vec<vk::ImageView>,
	depth_image: vk::Image,
	depth_view: vk::ImageView,
	depth_allocation: Option<Allocation>,
}

unsafe impl Send for SwapchainInner {}
unsafe impl Sync for SwapchainInner {}

impl SwapchainInner {
	fn destroy_resources(&mut self) {
		for view in self.image_views.drain(..) {
			unsafe { self.device.destroy_image_view(view, None); }
		}
		if self.depth_view != vk::ImageView::null() {
			unsafe { self.device.destroy_image_view(self.depth_view, None); }
			self.depth_view = vk::ImageView::null();
		}
		if self.depth_image != vk::Image::null() {
			unsafe { self.device.destroy_image(self.depth_image, None); }
			self.depth_image = vk::Image::null();
		}
		if let Some(alloc) = self.depth_allocation.take() {
			let _ = self.allocator.lock().unwrap().free(alloc);
		}
	}
}

impl Drop for SwapchainInner {
	fn drop(&mut self) {
		self.destroy_resources();
		if self.swapchain != vk::SwapchainKHR::null() {
			unsafe {
				self.swapchain_loader.destroy_swapchain(self.swapchain, None);
			}
			self.swapchain = vk::SwapchainKHR::null();
		}
	}
}

pub struct AshSwapchain {
	inner: Arc<Mutex<SwapchainInner>>,
	targets: Vec<AshRenderTarget>,
	render_finished_semaphores: Vec<AshSemaphore>,
}

unsafe impl Send for AshSwapchain {}
unsafe impl Sync for AshSwapchain {}

impl AshSwapchain {
	pub fn new(
		instance: &ash::Instance,
		device: &ash::Device,
		surface_loader: ash::khr::surface::Instance,
		physical_device: vk::PhysicalDevice,
		allocator: Arc<Mutex<Allocator>>,
		queue: vk::Queue,
		surface: vk::SurfaceKHR,
		width: u32,
		height: u32,
	) -> RhiResult<Self> {
		let swapchain_loader = ash::khr::swapchain::Device::new(instance, device);

		let caps = unsafe {
			surface_loader
				.get_physical_device_surface_capabilities(physical_device, surface)
				.map_err(|e| RhiError::BackendError(format!("Failed to get surface caps: {e}")))?
		};

		let formats = unsafe {
			surface_loader
				.get_physical_device_surface_formats(physical_device, surface)
				.map_err(|e| RhiError::BackendError(format!("Failed to get surface formats: {e}")))?
		};

		let selected_format = formats
			.iter()
			.find(|f| {
				(f.format == vk::Format::B8G8R8A8_UNORM || f.format == vk::Format::R8G8B8A8_UNORM)
					&& f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
			})
			.cloned()
			.unwrap_or(formats[0]);

		let extent = if caps.current_extent.width != u32::MAX {
			caps.current_extent
		} else {
			vk::Extent2D {
				width: width.clamp(caps.min_image_extent.width, caps.max_image_extent.width),
				height: height.clamp(caps.min_image_extent.height, caps.max_image_extent.height),
			}
		};

		let mut image_count = caps.min_image_count.max(3);
		if caps.max_image_count > 0 && image_count > caps.max_image_count {
			image_count = caps.max_image_count;
		}

		let present_modes = unsafe {
			surface_loader
				.get_physical_device_surface_present_modes(physical_device, surface)
				.unwrap_or_default()
		};

		let mut present_mode = vk::PresentModeKHR::FIFO;
		for mode in &present_modes {
			if *mode == vk::PresentModeKHR::MAILBOX {
				present_mode = *mode;
				break;
			}
		}

		let create_info = vk::SwapchainCreateInfoKHR::default()
			.surface(surface)
			.min_image_count(image_count)
			.image_format(selected_format.format)
			.image_color_space(selected_format.color_space)
			.image_extent(extent)
			.image_array_layers(1)
			.image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
			.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
			.pre_transform(caps.current_transform)
			.composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
			.present_mode(present_mode)
			.clipped(true);

		let swapchain = unsafe {
			swapchain_loader
				.create_swapchain(&create_info, None)
				.map_err(|e| RhiError::BackendError(format!("Failed to create swapchain: {e}")))?
		};

		let images = unsafe {
			swapchain_loader
				.get_swapchain_images(swapchain)
				.map_err(|e| RhiError::BackendError(format!("Failed to get swapchain images: {e}")))?
		};

		let depth_format = vk::Format::D32_SFLOAT;

		let mut inner = SwapchainInner {
			device: device.clone(),
			swapchain_loader,
			surface_loader,
			physical_device,
			allocator,
			queue,
			surface,
			swapchain,
			color_format: selected_format.format,
			depth_format,
			extent,
			images,
			image_views: Vec::new(),
			depth_image: vk::Image::null(),
			depth_view: vk::ImageView::null(),
			depth_allocation: None,
		};

		let (targets, semaphores) = Self::build_targets(&mut inner)?;

		println!("[mochi_rhi] Ash Vulkan swapchain created: {}x{}, {} images", inner.extent.width, inner.extent.height, targets.len());

		Ok(Self {
			inner: Arc::new(Mutex::new(inner)),
			targets,
			render_finished_semaphores: semaphores,
		})
	}

	fn build_targets(inner: &mut SwapchainInner) -> RhiResult<(Vec<AshRenderTarget>, Vec<AshSemaphore>)> {
		let mut targets = Vec::new();
		let mut semaphores = Vec::new();

		// 1. Swapchain color image views
		for img in &inner.images {
			let subresource_range = vk::ImageSubresourceRange::default()
				.aspect_mask(vk::ImageAspectFlags::COLOR)
				.base_mip_level(0)
				.level_count(1)
				.base_array_layer(0)
				.layer_count(1);

			let view_info = vk::ImageViewCreateInfo::default()
				.image(*img)
				.view_type(vk::ImageViewType::TYPE_2D)
				.format(inner.color_format)
				.subresource_range(subresource_range);

			let view = unsafe {
				inner
					.device
					.create_image_view(&view_info, None)
					.map_err(|e| RhiError::BackendError(format!("Failed to create image view: {e}")))?
			};
			inner.image_views.push(view);
		}

		// 2. Depth image
		let depth_extent = vk::Extent3D {
			width: inner.extent.width,
			height: inner.extent.height,
			depth: 1,
		};

		let depth_image_info = vk::ImageCreateInfo::default()
			.image_type(vk::ImageType::TYPE_2D)
			.format(inner.depth_format)
			.extent(depth_extent)
			.mip_levels(1)
			.array_layers(1)
			.samples(vk::SampleCountFlags::TYPE_1)
			.tiling(vk::ImageTiling::OPTIMAL)
			.usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
			.sharing_mode(vk::SharingMode::EXCLUSIVE)
			.initial_layout(vk::ImageLayout::UNDEFINED);

		let depth_image = unsafe {
			inner
				.device
				.create_image(&depth_image_info, None)
				.map_err(|e| RhiError::BackendError(format!("Failed to create depth image: {e}")))?
		};

		let requirements = unsafe { inner.device.get_image_memory_requirements(depth_image) };
		let alloc_desc = AllocationCreateDesc {
			name: "Swapchain Depth Buffer",
			requirements,
			location: gpu_allocator::MemoryLocation::GpuOnly,
			linear: false,
			allocation_scheme: AllocationScheme::GpuAllocatorManaged,
		};

		let allocation = inner
			.allocator
			.lock()
			.unwrap()
			.allocate(&alloc_desc)
			.map_err(|e| RhiError::BackendError(format!("Failed to allocate depth memory: {e}")))?;

		unsafe {
			inner
				.device
				.bind_image_memory(depth_image, allocation.memory(), allocation.offset())
				.map_err(|e| RhiError::BackendError(format!("Failed to bind depth memory: {e}")))?;
		}

		let depth_subresource = vk::ImageSubresourceRange::default()
			.aspect_mask(vk::ImageAspectFlags::DEPTH)
			.base_mip_level(0)
			.level_count(1)
			.base_array_layer(0)
			.layer_count(1);

		let depth_view_info = vk::ImageViewCreateInfo::default()
			.image(depth_image)
			.view_type(vk::ImageViewType::TYPE_2D)
			.format(inner.depth_format)
			.subresource_range(depth_subresource);

		let depth_view = unsafe {
			inner
				.device
				.create_image_view(&depth_view_info, None)
				.map_err(|e| RhiError::BackendError(format!("Failed to create depth image view: {e}")))?
		};

		inner.depth_image = depth_image;
		inner.depth_view = depth_view;
		inner.depth_allocation = Some(allocation);

		// 3. RenderTargets & Semaphores
		for i in 0..inner.images.len() {
			let sem = unsafe {
				inner
					.device
					.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
					.map_err(|e| RhiError::BackendError(format!("Failed to create semaphore: {e}")))?
			};
			semaphores.push(AshSemaphore::new(&inner.device, sem));

			let target = AshRenderTarget {
				color_image: inner.images[i],
				color_view: inner.image_views[i],
				color_format: from_vk_format(inner.color_format),
				depth_image,
				depth_view,
				depth_format: from_vk_format(inner.depth_format),
				extent: inner.extent,
				final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
			};
			targets.push(target);
		}

		Ok((targets, semaphores))
	}
}

impl RhiSwapchain<AshVulkanBackend> for AshSwapchain {
	fn resize(&mut self, width: u32, height: u32) -> RhiResult<()> {
		let mut inner = self.inner.lock().unwrap();
		unsafe { inner.device.device_wait_idle().unwrap(); }

		inner.destroy_resources();

		let old_swapchain = inner.swapchain;
		let caps = unsafe {
			inner
				.surface_loader
				.get_physical_device_surface_capabilities(inner.physical_device, inner.surface)
				.map_err(|e| RhiError::BackendError(format!("{e}")))?
		};

		let extent = if caps.current_extent.width != u32::MAX {
			caps.current_extent
		} else {
			vk::Extent2D {
				width: width.clamp(caps.min_image_extent.width, caps.max_image_extent.width),
				height: height.clamp(caps.min_image_extent.height, caps.max_image_extent.height),
			}
		};
		inner.extent = extent;

		let mut image_count = caps.min_image_count.max(3);
		if caps.max_image_count > 0 && image_count > caps.max_image_count {
			image_count = caps.max_image_count;
		}

		let create_info = vk::SwapchainCreateInfoKHR::default()
			.surface(inner.surface)
			.min_image_count(image_count)
			.image_format(inner.color_format)
			.image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
			.image_extent(extent)
			.image_array_layers(1)
			.image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
			.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
			.pre_transform(caps.current_transform)
			.composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
			.present_mode(vk::PresentModeKHR::FIFO)
			.clipped(true)
			.old_swapchain(old_swapchain);

		let new_swapchain = unsafe {
			inner
				.swapchain_loader
				.create_swapchain(&create_info, None)
				.map_err(|e| RhiError::BackendError(format!("Failed to recreate swapchain: {e}")))?
		};

		unsafe {
			inner.swapchain_loader.destroy_swapchain(old_swapchain, None);
		}
		inner.swapchain = new_swapchain;

		inner.images = unsafe {
			inner
				.swapchain_loader
				.get_swapchain_images(new_swapchain)
				.map_err(|e| RhiError::BackendError(format!("{e}")))?
		};

		let (targets, semaphores) = Self::build_targets(&mut inner)?;
		self.targets = targets;
		self.render_finished_semaphores = semaphores;

		Ok(())
	}

	fn acquire_next_image(&mut self, signal_sem: Option<&AshSemaphore>) -> RhiResult<u32> {
		let inner = self.inner.lock().unwrap();
		let sem = signal_sem.map(|s| s.handle()).unwrap_or(vk::Semaphore::null());
		let (index, _is_suboptimal) = unsafe {
			inner
				.swapchain_loader
				.acquire_next_image(inner.swapchain, u64::MAX, sem, vk::Fence::null())
				.map_err(|e| match e {
					vk::Result::ERROR_OUT_OF_DATE_KHR => RhiError::SwapchainOutOfDate,
					vk::Result::ERROR_SURFACE_LOST_KHR => RhiError::SwapchainLost,
					other => RhiError::BackendError(format!("acquire_next_image error: {other}")),
				})?
		};
		Ok(index)
	}

	fn present(&mut self, image_index: u32, wait_sem: Option<&AshSemaphore>) -> RhiResult<()> {
		let inner = self.inner.lock().unwrap();
		let sem_handles = match wait_sem {
			Some(s) => vec![s.handle()],
			None => vec![],
		};
		let swapchains = [inner.swapchain];
		let image_indices = [image_index];

		let present_info = vk::PresentInfoKHR::default()
			.wait_semaphores(&sem_handles)
			.swapchains(&swapchains)
			.image_indices(&image_indices);

		unsafe {
			inner
				.swapchain_loader
				.queue_present(inner.queue, &present_info)
				.map_err(|e| match e {
					vk::Result::ERROR_OUT_OF_DATE_KHR => RhiError::SwapchainOutOfDate,
					vk::Result::ERROR_SURFACE_LOST_KHR => RhiError::SwapchainLost,
					other => RhiError::BackendError(format!("queue_present error: {other}")),
				})?;
		}
		Ok(())
	}

	fn image_count(&self) -> u32 {
		self.targets.len() as u32
	}

	fn render_target(&self, index: u32) -> Option<&AshRenderTarget> {
		self.targets.get(index as usize)
	}

	fn render_finished_semaphore(&self, index: u32) -> &AshSemaphore {
		&self.render_finished_semaphores[index as usize]
	}
}
