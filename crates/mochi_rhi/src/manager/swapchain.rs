/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;

use crate::ffi::*;
pub use crate::sync::Semaphore;
use crate::manager::DeviceInner;
use crate::Device;


pub struct Surface {
	device: Arc<DeviceInner>,
	handle: u64,
}

unsafe impl Send for Surface {}
unsafe impl Sync for Surface {}

impl Surface {
	pub fn new(device: Arc<DeviceInner>, handle: u64) -> Self {
		Self { device, handle }
	}

	pub fn handle(&self) -> u64 {
		self.handle
	}
}

impl Drop for Surface {
	fn drop(&mut self) {
		if self.handle != 0 && !self.device.handle().is_null() {
			unsafe {
				mochi_rhi_destroy_surface(self.device.handle(), self.handle);
			}
			self.handle = 0;
		}
	}
}


pub struct SwapchainInner {
	device: Arc<DeviceInner>,
	handle: MochiSwapchainHandle,
}

unsafe impl Send for SwapchainInner {}
unsafe impl Sync for SwapchainInner {}

impl SwapchainInner {
	pub fn handle(&self) -> MochiSwapchainHandle {
		self.handle
	}

	pub fn device(&self) -> &Arc<DeviceInner> {
		&self.device
	}
}

impl Drop for SwapchainInner {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_swapchain_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}


pub struct Swapchain {
	inner: Arc<SwapchainInner>,
}

unsafe impl Send for Swapchain {}
unsafe impl Sync for Swapchain {}

impl Swapchain {
	pub fn new(device: &Device, surface: &Surface, width: u32, height: u32) -> Self {
		let handle = unsafe { mochi_rhi_swapchain_new(device.handle(), surface.handle, width, height) };
		assert!(!handle.is_null(), "RHI driver failed to create Swapchain");
		Self {
			inner: Arc::new(SwapchainInner {
				device: device.inner().clone(),
				handle,
			}),
		}
	}

	pub fn inner(&self) -> &Arc<SwapchainInner> {
		&self.inner
	}

	pub fn handle(&self) -> MochiSwapchainHandle {
		self.inner.handle
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		if !self.inner.handle.is_null() {
			unsafe { mochi_rhi_swapchain_resize(self.inner.handle, width, height); }
		}
	}

	pub fn acquire_next_image(&mut self) -> u32 {
		unsafe { mochi_rhi_swapchain_acquire_next_image(self.inner.handle, std::ptr::null_mut()) }
	}

	pub fn acquire_next_image_with_semaphore(&mut self, signal_semaphore: Option<&Semaphore>) -> u32 {
		let sem = signal_semaphore.map(|s| s.0).unwrap_or(std::ptr::null_mut());
		unsafe { mochi_rhi_swapchain_acquire_next_image(self.inner.handle, sem) }
	}

	pub fn present(&mut self, image_index: u32) {
		unsafe { mochi_rhi_swapchain_present(self.inner.handle, image_index, std::ptr::null_mut()); }
	}

	pub fn present_with_semaphore(&mut self, image_index: u32, wait_semaphore: Option<&Semaphore>) {
		let sem = wait_semaphore.map(|s| s.0).unwrap_or(std::ptr::null_mut());
		unsafe { mochi_rhi_swapchain_present(self.inner.handle, image_index, sem); }
	}

	pub fn image_count(&self) -> u32 {
		unsafe { mochi_rhi_swapchain_get_image_count(self.inner.handle) }
	}

	
	pub fn render_target<'a>(&'a self, index: u32) -> Option<crate::render_target::RenderTarget<'a>> {
		let target = unsafe { mochi_rhi_swapchain_get_render_target(self.inner.handle, index) };
		if target.is_null() {
			None
		} else {
			Some(crate::render_target::RenderTarget::from_handle(target))
		}
	}

	pub fn render_finished_semaphore(&self, index: u32) -> Semaphore {
		let sem = unsafe { mochi_rhi_swapchain_get_render_finished_semaphore(self.inner.handle, index) };
		Semaphore(sem)
	}
}
