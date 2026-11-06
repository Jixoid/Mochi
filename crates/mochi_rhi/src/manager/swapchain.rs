/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{Device, ffi::*};


pub struct Surface {
	device: MochiDeviceHandle,
	handle: u64,
}

impl Surface {
	pub fn new(device: MochiDeviceHandle, handle: u64) -> Self {
		Self { device, handle }
	}

	pub fn handle(&self) -> u64 {
		self.handle
	}
}

impl Drop for Surface {
	fn drop(&mut self) {
		if self.handle != 0 && !self.device.is_null() {
			unsafe {
				mochi_rhi_destroy_surface(self.device, self.handle);
			}
			self.handle = 0;
		}
	}
}


pub struct Swapchain {
	handle: MochiSwapchainHandle,
}

impl Swapchain {
	pub fn new(device: &Device, surface: &Surface, width: u32, height: u32) -> Self {
		let handle = unsafe { mochi_rhi_swapchain_new(device.handle(), surface.handle, width, height) };
		assert!(!handle.is_null(), "RHI driver failed to create Swapchain");
		Self { handle }
	}
	
	pub fn handle(&self) -> MochiSwapchainHandle {
		self.handle
	}


	pub fn resize(&mut self, width: u32, height: u32) {
		if !self.handle.is_null() {
			unsafe { mochi_rhi_swapchain_resize(self.handle, width, height); }
		}
	}

	pub fn acquire_next_image(&self) -> u32 {
		unsafe { mochi_rhi_swapchain_acquire_next_image(self.handle, std::ptr::null_mut()) }
	}

	pub fn acquire_next_image_with_semaphore(&self, signal_semaphore: *mut std::ffi::c_void) -> u32 {
		unsafe { mochi_rhi_swapchain_acquire_next_image(self.handle, signal_semaphore) }
	}

	pub fn present(&self, image_index: u32) {
		unsafe { mochi_rhi_swapchain_present(self.handle, image_index, std::ptr::null_mut()); }
	}

	pub fn present_with_semaphore(&self, image_index: u32, wait_semaphore: *mut std::ffi::c_void) {
		unsafe { mochi_rhi_swapchain_present(self.handle, image_index, wait_semaphore); }
	}

	pub fn image_count(&self) -> u32 {
		unsafe { mochi_rhi_swapchain_get_image_count(self.handle) }
	}

	pub fn render_target(&self, index: u32) -> Option<crate::render_target::RenderTarget> {
		let target = unsafe { mochi_rhi_swapchain_get_render_target(self.handle, index) };
		if target.is_null() {
			None
		} else {
			Some(crate::render_target::RenderTarget::from_handle(target))
		}
	}

	pub fn render_finished_semaphore(&self, index: u32) -> *mut std::ffi::c_void {
		unsafe { mochi_rhi_swapchain_get_render_finished_semaphore(self.handle, index) }
	}
}

impl Drop for Swapchain {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_swapchain_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}
