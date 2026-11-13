/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod allocator;
pub mod buffer;
pub mod command;
pub mod device;
pub mod image;
pub mod pipeline;
pub mod sampler;
pub mod shader;
pub mod swapchain;
pub mod sync;
pub mod types;

pub use allocator::*;
pub use buffer::*;
pub use command::*;
pub use device::*;
pub use image::*;
pub use pipeline::*;
pub use sampler::*;
pub use shader::*;
pub use swapchain::*;
pub use sync::*;
pub use types::*;

use ash::vk::Handle;
use mochi_rhi::error::RhiResult;
use mochi_rhi::traits::{RhiBackend, RhiDevice};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AshVulkanBackend;

impl RhiBackend for AshVulkanBackend {
	type Device = AshDevice;
	type Allocator = AshAllocator;
	type Buffer = AshBuffer;
	type Image2 = AshImage2;
	type ImageView2 = AshImageView2;
	type Sampler = AshSampler;
	type Shader = AshShader;
	type Pipeline = AshPipeline;
	type PipelineMeta = AshPipelineMeta;
	type CommandBuffer = AshCommandBuffer;
	type CommandQueue = AshCommandQueue;
	type Swapchain = AshSwapchain;
	type RenderTarget = AshRenderTarget;
	type Semaphore = AshSemaphore;
	type Fence = AshFence;

	fn create_device(app_name: &str, major: u16, minor: u16, patch: u16, build: u16) -> RhiResult<Self::Device> {
		AshDevice::new(app_name, major, minor, patch, build)
	}

	fn create_allocator(device: &Self::Device) -> RhiResult<Self::Allocator> {
		Ok(AshAllocator::new(device.device(), device.allocator().clone()))
	}

	fn create_command_queue(device: &Self::Device) -> RhiResult<Self::CommandQueue> {
		AshCommandQueue::new(device.device(), device.main_queue_family(), device.queue())
	}

	fn create_swapchain(device: &Self::Device, surface: u64, width: u32, height: u32) -> RhiResult<Self::Swapchain> {
		let surface_khr = ash::vk::SurfaceKHR::from_raw(surface);
		AshSwapchain::new(
			device.instance(),
			device.device(),
			device.surface_loader().clone(),
			device.physical_device(),
			device.allocator().clone(),
			device.queue(),
			surface_khr,
			width,
			height,
		)
	}
}
