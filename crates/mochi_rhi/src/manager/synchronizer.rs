/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use crate::ffi::*;
pub use crate::sync::{Fence, Semaphore};
use crate::manager::DeviceInner;
use crate::Device;


pub struct SynchronizerInner {
	device: Arc<DeviceInner>,
	handle: MochiSynchronizerHandle,
}

unsafe impl Send for SynchronizerInner {}
unsafe impl Sync for SynchronizerInner {}

impl SynchronizerInner {
	pub fn handle(&self) -> MochiSynchronizerHandle {
		self.handle
	}

	pub fn device(&self) -> &Arc<DeviceInner> {
		&self.device
	}
}

impl Drop for SynchronizerInner {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_synchronizer_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}

#[derive(Clone)]
pub struct Synchronizer {
	inner: Arc<SynchronizerInner>,
}

unsafe impl Send for Synchronizer {}
unsafe impl Sync for Synchronizer {}

impl Synchronizer {
	pub fn new(device: &Device, max_frames_in_flight: u32) -> Self {
		let handle = unsafe { mochi_rhi_synchronizer_new(device.handle(), max_frames_in_flight) };
		assert!(!handle.is_null(), "RHI driver failed to create Synchronizer");
		Self {
			inner: Arc::new(SynchronizerInner {
				device: device.inner().clone(),
				handle,
			}),
		}
	}

	pub fn inner(&self) -> &Arc<SynchronizerInner> {
		&self.inner
	}

	pub fn handle(&self) -> MochiSynchronizerHandle {
		self.inner.handle
	}

	pub fn begin_frame(&mut self) {
		unsafe { mochi_rhi_synchronizer_begin_frame(self.inner.handle) }
	}

	pub fn end_frame(&mut self) {
		unsafe { mochi_rhi_synchronizer_end_frame(self.inner.handle) }
	}

	pub fn current_frame(&self) -> u32 {
		unsafe { mochi_rhi_synchronizer_current_frame(self.inner.handle) }
	}

	pub fn image_available_semaphore(&self) -> Semaphore {
		let ptr = unsafe { mochi_rhi_synchronizer_get_image_available_semaphore(self.inner.handle) };
		Semaphore(ptr)
	}

	pub fn in_flight_fence(&self) -> Fence {
		let ptr = unsafe { mochi_rhi_synchronizer_get_in_flight_fence(self.inner.handle) };
		Fence(ptr)
	}
}
