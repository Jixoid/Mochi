/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;

use crate::ffi::*;
use crate::manager::DeviceInner;
use crate::{Buffer, Device, Image2};


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum TransferTime {
	Now = 0,
	Lazy = 1,
}

pub struct UploaderInner {
	device: Arc<DeviceInner>,
	handle: MochiUploaderHandle,
}

unsafe impl Send for UploaderInner {}
unsafe impl Sync for UploaderInner {}

impl UploaderInner {
	pub fn handle(&self) -> MochiUploaderHandle {
		self.handle
	}

	pub fn device(&self) -> &Arc<DeviceInner> {
		&self.device
	}
}

impl Drop for UploaderInner {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_uploader_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}

#[derive(Clone)]
pub struct Uploader {
	inner: Arc<UploaderInner>,
}

unsafe impl Send for Uploader {}
unsafe impl Sync for Uploader {}

impl Uploader {
	pub fn new(device: &Device) -> Self {
		let handle = unsafe { mochi_rhi_uploader_new(device.handle()) };
		assert!(!handle.is_null(), "RHI driver failed to create Uploader");
		Self {
			inner: Arc::new(UploaderInner {
				device: device.inner().clone(),
				handle,
			}),
		}
	}

	pub fn inner(&self) -> &Arc<UploaderInner> {
		&self.inner
	}

	pub fn handle(&self) -> MochiUploaderHandle {
		self.inner.handle
	}

	pub fn copy_memory_to_buffer(&self, time: TransferTime, src: &[u8], dst: &mut Buffer) {
		assert!(
			(src.len() as u64) <= dst.size(),
			"Uploader: Source slice ({} bytes) exceeds destination buffer capacity ({} bytes)",
			src.len(),
			dst.size()
		);
		unsafe {
			mochi_rhi_uploader_copy_memory_to_buffer(
				self.inner.handle,
				time as u32,
				src.as_ptr() as *const std::ffi::c_void,
				dst.handle(),
			);
		}
	}

	pub fn copy_memory_to_image(&self, time: TransferTime, src: &[u8], dst: &mut Image2) {
		assert!(!src.is_empty(), "Uploader: Cannot copy empty slice to Image2");
		let ext = dst.ext();
		assert!(ext[0] > 0 && ext[1] > 0, "Uploader: Cannot copy to 0-sized Image2 (extent: {:?})", ext);
		assert!(
			(src.len() as u64) >= (ext[0] as u64 * ext[1] as u64),
			"Uploader: Source slice ({} bytes) is smaller than image dimension pixels ({}x{})",
			src.len(),
			ext[0],
			ext[1]
		);
		unsafe {
			mochi_rhi_uploader_copy_memory_to_image(
				self.inner.handle,
				time as u32,
				src.as_ptr() as *const std::ffi::c_void,
				dst.handle(),
			);
		}
	}
}
