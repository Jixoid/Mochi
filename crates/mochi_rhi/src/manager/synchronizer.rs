/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/

use crate::ffi::*;
use crate::Device;

pub struct Synchronizer {
	handle: MochiSynchronizerHandle,
}

impl Synchronizer {
	pub fn new(device: &Device, max_frames_in_flight: u32) -> Self {
		let handle = unsafe { mochi_rhi_synchronizer_new(device.handle(), max_frames_in_flight) };
		assert!(!handle.is_null(), "RHI driver failed to create Synchronizer");
		Self { handle }
	}

	pub fn handle(&self) -> MochiSynchronizerHandle {
		self.handle
	}

	
	pub fn begin_frame(&self) {
		unsafe { mochi_rhi_synchronizer_begin_frame(self.handle) }
	}

	pub fn end_frame(&self) {
		unsafe { mochi_rhi_synchronizer_end_frame(self.handle) }
	}

	pub fn current_frame(&self) -> u32 {
		unsafe { mochi_rhi_synchronizer_current_frame(self.handle) }
	}

	pub fn image_available_semaphore(&self) -> *mut std::ffi::c_void {
		unsafe { mochi_rhi_synchronizer_get_image_available_semaphore(self.handle) }
	}

	pub fn in_flight_fence(&self) -> *mut std::ffi::c_void {
		unsafe { mochi_rhi_synchronizer_get_in_flight_fence(self.handle) }
	}
}

impl Drop for Synchronizer {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_synchronizer_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}
