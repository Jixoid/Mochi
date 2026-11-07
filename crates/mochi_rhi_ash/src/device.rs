/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ffi::CString;
use std::sync::{Arc, Mutex};
use ash::vk::{self, Handle};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::allocator::AshAllocator;
use crate::buffer::AshBuffer;
use crate::image::AshImageView;
use crate::pipeline::{AshPipeline, AshPipelineMeta};
use crate::sampler::AshSampler;
use crate::shader::AshShader;
use crate::sync::{AshFence, AshSemaphore};
use crate::AshVulkanBackend;
use mochi_rhi::error::{RhiError, RhiResult};
use mochi_rhi::format::Format;
use mochi_rhi::pipeline::{PolygonMode, PrimitiveTopology};
use mochi_rhi::shader::ShaderStage;
use mochi_rhi::traits::RhiDevice;
use mochi_except::*;


pub struct DeviceInner {
	entry: ash::Entry,
	surface_loader: Option<ash::khr::surface::Instance>,
	allocator: Option<Arc<Mutex<Allocator>>>,
	device: Option<ash::Device>,
	instance: Option<ash::Instance>,
	physical_device: vk::PhysicalDevice,
	queue: vk::Queue,
	queue_family: u32,
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
}

impl Drop for DeviceInner {
	fn drop(&mut self) {
		unsafe {
			if let Some(device) = &self.device {
				let _ = device.device_wait_idle();
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
			.buffer_device_address(true);

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
		mochi_disp!("Ash Vulkan device initialized: {}", dev_name);

		let inner = DeviceInner {
			entry,
			instance: Some(instance),
			physical_device,
			device: Some(device),
			queue,
			queue_family,
			allocator: Some(Arc::new(Mutex::new(allocator))),
			surface_loader: Some(surface_loader),
		};

		Ok(Self {
			inner: Arc::new(inner),
		})
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
		AshPipelineMeta::create(self.inner.device(), push_stages, push_size)
	}

	pub fn create_graphics_pipeline(
		&self,
		meta: &AshPipelineMeta,
		shaders: &[&AshShader],
		polymode: PolygonMode,
		primitive_topology: PrimitiveTopology,
		color_format: Format,
		depth_format: Format,
	) -> RhiResult<AshPipeline> {
		AshPipeline::create_graphics(
			self.inner.device(),
			meta,
			shaders,
			polymode,
			primitive_topology,
			color_format,
			depth_format,
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

	fn write_texture_descriptor(&self, _view: &AshImageView, _sampler: &AshSampler) -> u32 {
		0
	}

	fn descriptor_heap(&self) -> Option<AshBuffer> {
		None
	}

	fn sampler_heap(&self) -> Option<AshBuffer> {
		None
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
	) -> RhiResult<AshPipeline> {
		self.create_graphics_pipeline(meta, shaders, polymode, primitive_topology, color_format, depth_format)
	}
}
