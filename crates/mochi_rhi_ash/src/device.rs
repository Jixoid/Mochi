/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ffi::CString;
use std::sync::{Arc, Mutex};
use ash::vk::{self, Handle, PhysicalDeviceType};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::allocator::AshAllocator;
use crate::buffer::AshBuffer;
use crate::command::AshCommandQueue;
use crate::image::{AshImage, AshImageView};
use crate::pipeline::{AshPipeline, AshPipelineMeta};
use crate::sampler::AshSampler;
use crate::shader::AshShader;
use crate::swapchain::AshRenderTarget;
use crate::sync::{AshFence, AshSemaphore};
use crate::types::{to_vk_address_mode, to_vk_filter, to_vk_format};
use crate::AshVulkanBackend;
use mochi_rhi::error::{RhiError, RhiResult};
use mochi_rhi::format::Format;
use mochi_rhi::pipeline::{PolygonMode, PrimitiveTopology};
use mochi_rhi::sampler::{SamplerAddressMode, SamplerFilter};
use mochi_rhi::shader::ShaderStage;
use mochi_rhi::traits::RhiDevice;
use mochi_except::*;


pub struct AshBindlessHeap {
	pub layout: vk::DescriptorSetLayout,
	pub pool: vk::DescriptorPool,
	pub set: vk::DescriptorSet,
	pub next_id: Mutex<u32>,
}

pub struct DeviceInner {
	entry: ash::Entry,
	surface_loader: Option<ash::khr::surface::Instance>,
	allocator: Option<Arc<Mutex<Allocator>>>,
	device: Option<ash::Device>,
	instance: Option<ash::Instance>,
	physical_device: vk::PhysicalDevice,
	queue: vk::Queue,
	queue_family: u32,
	bindless_heap: Option<AshBindlessHeap>,
}

unsafe impl Send for DeviceInner {}
unsafe impl Sync for DeviceInner {}

impl DeviceInner {
	pub fn device(&self) -> &ash::Device {
		self.device.as_ref().expect("Ash logical device already destroyed")
	}

	pub fn instance(&self) -> &ash::Instance {
		self.instance.as_ref().expect("Ash instance already destroyed")
	}

	pub fn surface_loader(&self) -> &ash::khr::surface::Instance {
		self.surface_loader.as_ref().expect("Ash surface loader already destroyed")
	}

	pub fn allocator(&self) -> &Arc<Mutex<Allocator>> {
		self.allocator.as_ref().expect("Ash allocator already destroyed")
	}

	pub fn bindless_layout(&self) -> Option<vk::DescriptorSetLayout> {
		self.bindless_heap.as_ref().map(|h| h.layout)
	}

	pub fn bindless_set(&self) -> Option<vk::DescriptorSet> {
		self.bindless_heap.as_ref().map(|h| h.set)
	}
}

impl Drop for DeviceInner {
	fn drop(&mut self) {
		unsafe {
			if let Some(device) = &self.device {
				let _ = device.device_wait_idle();
				if let Some(heap) = &self.bindless_heap {
					device.destroy_descriptor_pool(heap.pool, None);
					device.destroy_descriptor_set_layout(heap.layout, None);
				}
			}
			drop(self.allocator.take());
			if let Some(device) = self.device.take() {
				device.destroy_device(None);
			}
			drop(self.surface_loader.take());
			if let Some(instance) = self.instance.take() {
				instance.destroy_instance(None);
			}
		}
	}
}

#[derive(Clone)]
pub struct AshDevice {
	inner: Arc<DeviceInner>,
}

unsafe impl Send for AshDevice {}
unsafe impl Sync for AshDevice {}

impl AshDevice {
	pub fn new(app_name: &str, major: u16, minor: u16, patch: u16, _build: u16) -> RhiResult<Self> {
		let entry = unsafe { ash::Entry::load().map_err(|e| RhiError::InitializationFailed(format!("{e}")))? };

		let app_cname = CString::new(app_name).map_err(|e| RhiError::InitializationFailed(e.to_string()))?;
		let app_info = vk::ApplicationInfo::default()
			.application_name(&app_cname)
			.application_version(vk::make_api_version(0, major as u32, minor as u32, patch as u32))
			.engine_name(c"Mochi")
			.engine_version(vk::make_api_version(0, 0, 1, 0))
			.api_version(vk::API_VERSION_1_3);

		let mut extension_names = vec![
			ash::khr::surface::NAME.as_ptr(),
		];

		#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
		{
			extension_names.push(ash::khr::wayland_surface::NAME.as_ptr());
			extension_names.push(ash::khr::xlib_surface::NAME.as_ptr());
			extension_names.push(ash::khr::xcb_surface::NAME.as_ptr());
		}

		#[cfg(windows)]
		{
			extension_names.push(ash::khr::win32_surface::NAME.as_ptr());
		}

		let supported_inst_exts = unsafe {
			entry
				.enumerate_instance_extension_properties(None)
				.unwrap_or_default()
		};
		let mut enabled_inst_extensions = Vec::new();
		for req in &extension_names {
			let req_cstr = unsafe { std::ffi::CStr::from_ptr(*req) };
			if supported_inst_exts.iter().any(|ext| {
				let name = unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()) };
				name == req_cstr
			}) {
				enabled_inst_extensions.push(*req);
			}
		}

		let instance_create_info = vk::InstanceCreateInfo::default()
			.application_info(&app_info)
			.enabled_extension_names(&enabled_inst_extensions);

		let instance = unsafe {
			entry
				.create_instance(&instance_create_info, None)
				.map_err(|e| RhiError::InitializationFailed(format!("Failed to create Vulkan instance: {e}")))?
		};

		let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

		let physical_devices = unsafe {
			instance
				.enumerate_physical_devices()
				.map_err(|e| RhiError::InitializationFailed(format!("{e}")))?
		};

		let mut selected_phys = None;
		let mut selected_q_family = None;

		// Prioritize discrete GPU, then integrated GPU
		for phys in &physical_devices {
			let props = unsafe { instance.get_physical_device_properties(*phys) };
			if props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
				let q_props = unsafe { instance.get_physical_device_queue_family_properties(*phys) };
				for (idx, q) in q_props.iter().enumerate() {
					if q.queue_flags.contains(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE) {
						selected_phys = Some(*phys);
						selected_q_family = Some(idx as u32);
						break;
					}
				}
				if selected_phys.is_some() {
					break;
				}
			}
		}

		if selected_phys.is_none() {
			for phys in &physical_devices {
				let q_props = unsafe { instance.get_physical_device_queue_family_properties(*phys) };
				for (idx, q) in q_props.iter().enumerate() {
					if q.queue_flags.contains(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE) {
						selected_phys = Some(*phys);
						selected_q_family = Some(idx as u32);
						break;
					}
				}
				if selected_phys.is_some() {
					break;
				}
			}
		}

		let physical_device = selected_phys
			.ok_or_else(|| RhiError::InitializationFailed("No suitable Vulkan physical device found".into()))?;
		let queue_family = selected_q_family.unwrap();

		let queue_priorities = [1.0f32];
		let queue_create_info = vk::DeviceQueueCreateInfo::default()
			.queue_family_index(queue_family)
			.queue_priorities(&queue_priorities);

		let queue_create_infos = [queue_create_info];

		let desired_device_extensions = [
			ash::khr::swapchain::NAME.as_ptr(),
			ash::khr::dynamic_rendering::NAME.as_ptr(),
			ash::khr::synchronization2::NAME.as_ptr(),
		];

		let supported_dev_exts = unsafe {
			instance
				.enumerate_device_extension_properties(physical_device)
				.unwrap_or_default()
		};
		let mut enabled_dev_extensions = Vec::new();
		for req in &desired_device_extensions {
			let req_cstr = unsafe { std::ffi::CStr::from_ptr(*req) };
			if supported_dev_exts.iter().any(|ext| {
				let name = unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()) };
				name == req_cstr
			}) {
				enabled_dev_extensions.push(*req);
			}
		}

		let mut features12 = vk::PhysicalDeviceVulkan12Features::default()
			.buffer_device_address(true)
			.descriptor_indexing(true)
			.shader_sampled_image_array_non_uniform_indexing(true)
			.descriptor_binding_partially_bound(true)
			.descriptor_binding_variable_descriptor_count(true)
			.runtime_descriptor_array(true);

		let mut features13 = vk::PhysicalDeviceVulkan13Features::default()
			.dynamic_rendering(true)
			.synchronization2(true);

		let features = vk::PhysicalDeviceFeatures::default()
			.sampler_anisotropy(true)
			.fill_mode_non_solid(true)
			.shader_int64(true);

		let device_create_info = vk::DeviceCreateInfo::default()
			.queue_create_infos(&queue_create_infos)
			.enabled_extension_names(&enabled_dev_extensions)
			.enabled_features(&features)
			.push_next(&mut features12)
			.push_next(&mut features13);

		let device = unsafe {
			instance
				.create_device(physical_device, &device_create_info, None)
				.map_err(|e| RhiError::InitializationFailed(format!("Failed to create Vulkan logical device: {e}")))?
		};

		let queue = unsafe { device.get_device_queue(queue_family, 0) };

		let allocator_desc = AllocatorCreateDesc {
			instance: instance.clone(),
			device: device.clone(),
			physical_device,
			debug_settings: Default::default(),
			buffer_device_address: true,
			allocation_sizes: Default::default(),
		};

		let allocator = Allocator::new(&allocator_desc)
			.map_err(|e| RhiError::InitializationFailed(format!("Failed to create GPU allocator: {e}")))?;

		let props = unsafe { instance.get_physical_device_properties(physical_device) };
		let dev_name = unsafe { std::ffi::CStr::from_ptr(props.device_name.as_ptr()) }.to_string_lossy();
		let pdt = match props.device_type {
			PhysicalDeviceType::INTEGRATED_GPU => "integrated",
			PhysicalDeviceType::DISCRETE_GPU => "discrete",
			PhysicalDeviceType::CPU => "cpu",
			PhysicalDeviceType::VIRTUAL_GPU => "virtual",
			_ => "other",
		};
		mochi_disp!("Device initialized: {dev_name} ({pdt})");

		const MAX_BINDLESS_TEXTURES: u32 = 4096;

		let binding = vk::DescriptorSetLayoutBinding::default()
			.binding(0)
			.descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
			.descriptor_count(MAX_BINDLESS_TEXTURES)
			.stage_flags(vk::ShaderStageFlags::ALL);

		let binding_flags = [vk::DescriptorBindingFlags::PARTIALLY_BOUND
			| vk::DescriptorBindingFlags::UPDATE_AFTER_BIND];

		let mut binding_flags_info = vk::DescriptorSetLayoutBindingFlagsCreateInfo::default()
			.binding_flags(&binding_flags);

		let bindings = [binding];
		let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
			.flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
			.bindings(&bindings)
			.push_next(&mut binding_flags_info);

		let bindless_layout = unsafe {
			device
				.create_descriptor_set_layout(&layout_info, None)
				.map_err(|e| RhiError::InitializationFailed(format!("Failed to create bindless set layout: {e}")))?
		};

		let pool_size = [vk::DescriptorPoolSize::default()
			.ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
			.descriptor_count(MAX_BINDLESS_TEXTURES)];

		let pool_info = vk::DescriptorPoolCreateInfo::default()
			.flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
			.max_sets(1)
			.pool_sizes(&pool_size);

		let bindless_pool = unsafe {
			device
				.create_descriptor_pool(&pool_info, None)
				.map_err(|e| RhiError::InitializationFailed(format!("Failed to create bindless pool: {e}")))?
		};

		let set_layouts = [bindless_layout];
		let alloc_info = vk::DescriptorSetAllocateInfo::default()
			.descriptor_pool(bindless_pool)
			.set_layouts(&set_layouts);

		let bindless_sets = unsafe {
			device
				.allocate_descriptor_sets(&alloc_info)
				.map_err(|e| RhiError::InitializationFailed(format!("Failed to allocate bindless descriptor set: {e}")))?
		};

		let bindless_heap = AshBindlessHeap {
			layout: bindless_layout,
			pool: bindless_pool,
			set: bindless_sets[0],
			next_id: Mutex::new(0),
		};

		let inner = DeviceInner {
			entry,
			instance: Some(instance),
			physical_device,
			device: Some(device),
			queue,
			queue_family,
			allocator: Some(Arc::new(Mutex::new(allocator))),
			surface_loader: Some(surface_loader),
			bindless_heap: Some(bindless_heap),
		};

		Ok(Self {
			inner: Arc::new(inner),
		})
	}

	pub fn bindless_layout(&self) -> Option<vk::DescriptorSetLayout> {
		self.inner.bindless_layout()
	}

	pub fn bindless_set(&self) -> Option<vk::DescriptorSet> {
		self.inner.bindless_set()
	}

	pub fn device(&self) -> &ash::Device {
		self.inner.device()
	}

	pub fn instance(&self) -> &ash::Instance {
		self.inner.instance()
	}

	pub fn physical_device(&self) -> vk::PhysicalDevice {
		self.inner.physical_device
	}

	pub fn queue(&self) -> vk::Queue {
		self.inner.queue
	}

	pub fn allocator(&self) -> &Arc<Mutex<Allocator>> {
		self.inner.allocator()
	}

	pub fn entry(&self) -> &ash::Entry {
		&self.inner.entry
	}

	pub fn surface_loader(&self) -> &ash::khr::surface::Instance {
		self.inner.surface_loader()
	}

	pub fn create_shader(&self, stage: ShaderStage, code: &[u8], entry: &str) -> RhiResult<AshShader> {
		if code.len() % 4 != 0 {
			return Err(RhiError::ShaderCompilation(
				"SPIR-V code length must be a multiple of 4 bytes".into(),
			));
		}
		let words: Vec<u32> = if code.as_ptr() as usize % std::mem::align_of::<u32>() == 0 {
			unsafe {
				std::slice::from_raw_parts(code.as_ptr() as *const u32, code.len() / 4).to_vec()
			}
		} else {
			let mut words = vec![0u32; code.len() / 4];
			unsafe {
				std::ptr::copy_nonoverlapping(code.as_ptr(), words.as_mut_ptr() as *mut u8, code.len());
			}
			words
		};
		let info = vk::ShaderModuleCreateInfo::default().code(&words);
		let module = unsafe {
			self.inner
				.device()
				.create_shader_module(&info, None)
				.map_err(|e| RhiError::ShaderCompilation(format!("Failed to create shader module: {e}")))?
		};
		Ok(AshShader::new(self.inner.device(), module, stage, entry))
	}

	pub fn create_pipeline_meta(&self, push_stages: ShaderStage, push_size: u32) -> RhiResult<AshPipelineMeta> {
		AshPipelineMeta::create(self.inner.device(), self.inner.bindless_layout(), push_stages, push_size)
	}

	pub fn create_graphics_pipeline(
		&self,
		meta: &AshPipelineMeta,
		shaders: &[&AshShader],
		polymode: PolygonMode,
		primitive_topology: PrimitiveTopology,
		color_format: Format,
		depth_format: Format,
		blend_enable: bool,
		depth_write: bool,
		samples: mochi_rhi::MsaaSamples,
	) -> RhiResult<AshPipeline> {
		AshPipeline::create_graphics(
			self.inner.device(),
			meta,
			shaders,
			polymode,
			primitive_topology,
			color_format,
			depth_format,
			self.inner.bindless_set(),
			blend_enable,
			depth_write,
			samples,
		)
	}

	pub fn create_compute_pipeline(
		&self,
		meta: &AshPipelineMeta,
		shader: &AshShader,
	) -> RhiResult<AshPipeline> {
		AshPipeline::create_compute(
			self.inner.device(),
			meta,
			shader,
			self.inner.bindless_set(),
		)
	}
}

impl RhiDevice<AshVulkanBackend> for AshDevice {
	fn wait_idle(&self) -> RhiResult<()> {
		unsafe {
			self.inner
				.device()
				.device_wait_idle()
				.map_err(|e| RhiError::BackendError(format!("{e}")))?;
		}
		Ok(())
	}

	fn main_queue_family(&self) -> u32 {
		self.inner.queue_family
	}

	fn create_semaphore(&self) -> RhiResult<AshSemaphore> {
		let sem = unsafe {
			self.inner
				.device()
				.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
				.map_err(|e| RhiError::BackendError(format!("{e}")))?
		};
		Ok(AshSemaphore::new(self.inner.device(), sem))
	}

	fn create_fence(&self, signaled: bool) -> RhiResult<AshFence> {
		let mut info = vk::FenceCreateInfo::default();
		if signaled {
			info = info.flags(vk::FenceCreateFlags::SIGNALED);
		}
		let fence = unsafe {
			self.inner
				.device()
				.create_fence(&info, None)
				.map_err(|e| RhiError::BackendError(format!("{e}")))?
		};
		Ok(AshFence::new(self.inner.device(), fence))
	}

	fn wait_fence(&self, fence: &AshFence, timeout_ns: u64) -> RhiResult<()> {
		unsafe {
			self.inner
				.device()
				.wait_for_fences(&[fence.handle()], true, timeout_ns)
				.map_err(|e| match e {
					vk::Result::TIMEOUT => RhiError::Timeout,
					other => RhiError::BackendError(format!("{other}")),
				})?;
		}
		Ok(())
	}

	fn reset_fence(&self, fence: &AshFence) -> RhiResult<()> {
		unsafe {
			self.inner
				.device()
				.reset_fences(&[fence.handle()])
				.map_err(|e| RhiError::BackendError(format!("{e}")))?;
		}
		Ok(())
	}

	fn create_surface<W: HasWindowHandle + HasDisplayHandle>(&self, window: &W) -> RhiResult<u64> {
		let raw_disp = window
			.display_handle()
			.map_err(|e| RhiError::SurfaceCreation(e.to_string()))?
			.as_raw();

		let raw_win = window
			.window_handle()
			.map_err(|e| RhiError::SurfaceCreation(e.to_string()))?
			.as_raw();

		let surface = unsafe {
			ash_window::create_surface(&self.inner.entry, self.inner.instance(), raw_disp, raw_win, None)
				.map_err(|e| RhiError::SurfaceCreation(format!("Failed to create window surface: {e}")))?
		};

		Ok(surface.as_raw())
	}

	fn destroy_surface(&self, surface_handle: u64) {
		let surface = vk::SurfaceKHR::from_raw(surface_handle);
		if surface != vk::SurfaceKHR::null() {
			if let Some(loader) = &self.inner.surface_loader {
				unsafe {
					loader.destroy_surface(surface, None);
				}
			}
		}
	}

	fn init_descriptor_heap(&self, _allocator: &AshAllocator) -> RhiResult<()> {
		Ok(())
	}

	fn fini_descriptor_heap(&self) {}

	fn write_texture_descriptor(&self, view: &AshImageView, sampler: &AshSampler) -> u32 {
		if let Some(heap) = &self.inner.bindless_heap {
			let mut lock = heap.next_id.lock().unwrap();
			let id = *lock;
			*lock += 1;

			let image_info = [vk::DescriptorImageInfo::default()
				.sampler(sampler.handle())
				.image_view(view.handle())
				.image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];

			let write = [vk::WriteDescriptorSet::default()
				.dst_set(heap.set)
				.dst_binding(0)
				.dst_array_element(id)
				.descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
				.image_info(&image_info)];

			unsafe {
				self.inner.device().update_descriptor_sets(&write, &[]);
			}

			id
		} else {
			0
		}
	}

	fn descriptor_heap(&self) -> Option<AshBuffer> {
		None
	}

	fn sampler_heap(&self) -> Option<AshBuffer> {
		None
	}

	fn create_image_view(&self, image: &AshImage, format: Format) -> RhiResult<AshImageView> {
		let is_depth = format == Format::Depth32f
			|| format == Format::Depth24Stencil8
			|| format == Format::Depth16
			|| format == Format::Depth32fStencil8;

		let aspect_mask = if is_depth {
			vk::ImageAspectFlags::DEPTH
		} else {
			vk::ImageAspectFlags::COLOR
		};

		let subresource_range = vk::ImageSubresourceRange {
			aspect_mask,
			base_mip_level: 0,
			level_count: 1,
			base_array_layer: 0,
			layer_count: 1,
		};

		let view_info = vk::ImageViewCreateInfo::default()
			.image(image.handle())
			.view_type(vk::ImageViewType::TYPE_2D)
			.format(to_vk_format(format))
			.subresource_range(subresource_range);

		let view = unsafe {
			self.inner
				.device()
				.create_image_view(&view_info, None)
				.map_err(|e| RhiError::BackendError(format!("Failed to create image view: {e}")))?
		};

		Ok(AshImageView::new(self.inner.device(), view))
	}

	fn create_sampler(&self, filter: SamplerFilter, address_mode: SamplerAddressMode) -> RhiResult<AshSampler> {
		let vk_filter = to_vk_filter(filter);
		let vk_addr = to_vk_address_mode(address_mode);
		let is_clamp_to_border = address_mode == SamplerAddressMode::ClampToBorder;
		let border_color = if is_clamp_to_border {
			vk::BorderColor::FLOAT_OPAQUE_WHITE
		} else {
			vk::BorderColor::INT_OPAQUE_BLACK
		};
		let sampler_info = vk::SamplerCreateInfo::default()
			.mag_filter(vk_filter)
			.min_filter(vk_filter)
			.address_mode_u(vk_addr)
			.address_mode_v(vk_addr)
			.address_mode_w(vk_addr)
			.anisotropy_enable(!is_clamp_to_border)
			.max_anisotropy(if is_clamp_to_border { 1.0 } else { 16.0 })
			.border_color(border_color)
			.unnormalized_coordinates(false)
			.compare_enable(false)
			.compare_op(vk::CompareOp::ALWAYS)
			.mipmap_mode(vk::SamplerMipmapMode::LINEAR);

		let sampler = unsafe {
			self.inner
				.device()
				.create_sampler(&sampler_info, None)
				.map_err(|e| RhiError::ResourceCreation(format!("Failed to create sampler: {e}")))?
		};

		Ok(AshSampler::new(self.inner.device(), sampler))
	}

	fn create_render_target(
		&self,
		color_image: &AshImage,
		color_view: &AshImageView,
		color_format: Format,
		depth_image: &AshImage,
		depth_view: &AshImageView,
		depth_format: Format,
		width: u32,
		height: u32,
	) -> AshRenderTarget {
		AshRenderTarget::new_offscreen(
			color_image.handle(),
			color_view.handle(),
			color_format,
			depth_image.handle(),
			depth_view.handle(),
			depth_format,
			width,
			height,
		)
	}

	fn create_depth_render_target(
		&self,
		depth_image: &AshImage,
		depth_view: &AshImageView,
		depth_format: Format,
		width: u32,
		height: u32,
	) -> AshRenderTarget {
		AshRenderTarget::new_depth_only(
			depth_image.handle(),
			depth_view.handle(),
			depth_format,
			width,
			height,
		)
	}

	fn create_color_render_target(
		&self,
		color_image: &AshImage,
		color_view: &AshImageView,
		color_format: Format,
		width: u32,
		height: u32,
	) -> AshRenderTarget {
		AshRenderTarget::new_color_only(
			color_image.handle(),
			color_view.handle(),
			color_format,
			width,
			height,
		)
	}

	fn upload_image_data(
		&self,
		alloc: &AshAllocator,
		queue: &AshCommandQueue,
		image: &AshImage,
		width: u32,
		height: u32,
		data: &[u8],
	) -> RhiResult<()> {
		use mochi_rhi::traits::{RhiAllocator, RhiBuffer, RhiCommandQueue};

		let mut staging_buf = alloc.alloc_buffer(
			data.len() as u64,
			mochi_rhi::BufferUsage::TransferSrc,
			mochi_rhi::manager::AllocationCreate::HostSequentialWrite,
			mochi_rhi::manager::AllocationLocation::PreferHost,
		)?;
		let ptr = staging_buf.map()?;
		unsafe {
			std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
		}
		staging_buf.unmap();

		let cmd = queue.allocate_graphic_command()?;
		let begin_info = vk::CommandBufferBeginInfo::default()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

		let raw_dev = self.inner.device();
		unsafe {
			raw_dev
				.begin_command_buffer(cmd.handle(), &begin_info)
				.map_err(|e| RhiError::BackendError(format!("{e}")))?;

			let barrier_to_dst = vk::ImageMemoryBarrier::default()
				.old_layout(vk::ImageLayout::UNDEFINED)
				.new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
				.src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.image(image.handle())
				.subresource_range(vk::ImageSubresourceRange {
					aspect_mask: vk::ImageAspectFlags::COLOR,
					base_mip_level: 0,
					level_count: 1,
					base_array_layer: 0,
					layer_count: 1,
				})
				.src_access_mask(vk::AccessFlags::empty())
				.dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);

			raw_dev.cmd_pipeline_barrier(
				cmd.handle(),
				vk::PipelineStageFlags::TOP_OF_PIPE,
				vk::PipelineStageFlags::TRANSFER,
				vk::DependencyFlags::empty(),
				&[],
				&[],
				&[barrier_to_dst],
			);

			let region = vk::BufferImageCopy::default()
				.buffer_offset(0)
				.buffer_row_length(0)
				.buffer_image_height(0)
				.image_subresource(vk::ImageSubresourceLayers {
					aspect_mask: vk::ImageAspectFlags::COLOR,
					mip_level: 0,
					base_array_layer: 0,
					layer_count: 1,
				})
				.image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
				.image_extent(vk::Extent3D {
					width,
					height,
					depth: 1,
				});

			raw_dev.cmd_copy_buffer_to_image(
				cmd.handle(),
				staging_buf.handle(),
				image.handle(),
				vk::ImageLayout::TRANSFER_DST_OPTIMAL,
				&[region],
			);

			let barrier_to_shader = vk::ImageMemoryBarrier::default()
				.old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
				.new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
				.src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.image(image.handle())
				.subresource_range(vk::ImageSubresourceRange {
					aspect_mask: vk::ImageAspectFlags::COLOR,
					base_mip_level: 0,
					level_count: 1,
					base_array_layer: 0,
					layer_count: 1,
				})
				.src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
				.dst_access_mask(vk::AccessFlags::SHADER_READ);

			raw_dev.cmd_pipeline_barrier(
				cmd.handle(),
				vk::PipelineStageFlags::TRANSFER,
				vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::VERTEX_SHADER,
				vk::DependencyFlags::empty(),
				&[],
				&[],
				&[barrier_to_shader],
			);

			raw_dev
				.end_command_buffer(cmd.handle())
				.map_err(|e| RhiError::BackendError(format!("{e}")))?;
		}

		queue.submit(&cmd)?;
		self.wait_idle()?;

		Ok(())
	}

	fn create_shader(&self, stage: ShaderStage, code: &[u8], entry: &str) -> RhiResult<AshShader> {
		self.create_shader(stage, code, entry)
	}

	fn create_pipeline_meta(&self, push_stages: ShaderStage, push_size: u32) -> RhiResult<AshPipelineMeta> {
		self.create_pipeline_meta(push_stages, push_size)
	}

	fn create_graphics_pipeline(
		&self,
		meta: &AshPipelineMeta,
		shaders: &[&AshShader],
		polymode: PolygonMode,
		primitive_topology: PrimitiveTopology,
		color_format: Format,
		depth_format: Format,
		blend_enable: bool,
		depth_write: bool,
		samples: mochi_rhi::MsaaSamples,
	) -> RhiResult<AshPipeline> {
		self.create_graphics_pipeline(meta, shaders, polymode, primitive_topology, color_format, depth_format, blend_enable, depth_write, samples)
	}

	fn create_compute_pipeline(
		&self,
		meta: &AshPipelineMeta,
		shader: &AshShader,
	) -> RhiResult<AshPipeline> {
		self.create_compute_pipeline(meta, shader)
	}
}
