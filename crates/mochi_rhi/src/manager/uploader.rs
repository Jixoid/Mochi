/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::ffi::*;
use crate::{Buffer, Device, Image2};


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum TransferTime {
	Now = 0,
	Lazy = 1,
}

pub struct Uploader {
	handle: MochiUploaderHandle,
}

impl Uploader {
	pub fn new(device: &Device) -> Self {
		let handle = unsafe { mochi_rhi_uploader_new(device.handle()) };
		assert!(!handle.is_null(), "RHI driver failed to create Uploader");
		Self { handle }
	}

	pub fn handle(&self) -> MochiUploaderHandle {
		self.handle
	}

	
	pub fn copy_memory_to_buffer(&self, time: TransferTime, src: &[u8], dst: &Buffer) {
		unsafe {
			mochi_rhi_uploader_copy_memory_to_buffer(
				self.handle,
				time as u32,
				src.as_ptr() as *const std::ffi::c_void,
				dst.handle(),
			);
		}
	}

	pub fn copy_memory_to_image(&self, time: TransferTime, src: &[u8], dst: &Image2) {
		unsafe {
			mochi_rhi_uploader_copy_memory_to_image(
				self.handle,
				time as u32,
				src.as_ptr() as *const std::ffi::c_void,
				dst.handle(),
			);
		}
	}
}

impl Drop for Uploader {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_uploader_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}
