/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use ash::vk;


pub struct SemaphoreInner {
	device: ash::Device,
	handle: vk::Semaphore,
}

unsafe impl Send for SemaphoreInner {}
unsafe impl Sync for SemaphoreInner {}

impl Drop for SemaphoreInner {
	fn drop(&mut self) {
		if self.handle != vk::Semaphore::null() {
			unsafe {
				self.device.destroy_semaphore(self.handle, None);
			}
			self.handle = vk::Semaphore::null();
		}
	}
}

#[derive(Clone)]
pub struct AshSemaphore {
	inner: Arc<SemaphoreInner>,
}

unsafe impl Send for AshSemaphore {}
unsafe impl Sync for AshSemaphore {}

impl AshSemaphore {
	pub fn new(device: &ash::Device, handle: vk::Semaphore) -> Self {
		Self {
			inner: Arc::new(SemaphoreInner {
				device: device.clone(),
				handle,
			}),
		}
	}

	pub fn handle(&self) -> vk::Semaphore {
		self.inner.handle
	}
}


pub struct FenceInner {
	device: ash::Device,
	handle: vk::Fence,
}

unsafe impl Send for FenceInner {}
unsafe impl Sync for FenceInner {}

impl Drop for FenceInner {
	fn drop(&mut self) {
		if self.handle != vk::Fence::null() {
			unsafe {
				self.device.destroy_fence(self.handle, None);
			}
			self.handle = vk::Fence::null();
		}
	}
}

#[derive(Clone)]
pub struct AshFence {
	inner: Arc<FenceInner>,
}

unsafe impl Send for AshFence {}
unsafe impl Sync for AshFence {}

impl AshFence {
	pub fn new(device: &ash::Device, handle: vk::Fence) -> Self {
		Self {
			inner: Arc::new(FenceInner {
				device: device.clone(),
				handle,
			}),
		}
	}

	pub fn handle(&self) -> vk::Fence {
		self.inner.handle
	}
}
