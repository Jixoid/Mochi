/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};
use ash::vk;
use gpu_allocator::vulkan::{Allocation, Allocator};

use mochi_rhi::error::{RhiError, RhiResult};
use mochi_rhi::traits::RhiBuffer;


pub struct BufferInner {
	device: ash::Device,
	allocator: Option<Arc<Mutex<Allocator>>>,
	handle: vk::Buffer,
	allocation: Option<Allocation>,
	size: u64,
	device_address: u64,
}

unsafe impl Send for BufferInner {}
unsafe impl Sync for BufferInner {}

impl Drop for BufferInner {
	fn drop(&mut self) {
		if self.handle != vk::Buffer::null() {
			unsafe {
				self.device.destroy_buffer(self.handle, None);
			}
			self.handle = vk::Buffer::null();
		}
		if let (Some(allocator), Some(alloc)) = (&self.allocator, self.allocation.take()) {
			let mut lock = allocator.lock().unwrap();
			let _ = lock.free(alloc);
		}
	}
}

pub struct AshBuffer {
	inner: Arc<Mutex<BufferInner>>,
}

unsafe impl Send for AshBuffer {}
unsafe impl Sync for AshBuffer {}

impl AshBuffer {
	pub fn new(
		device: &ash::Device,
		allocator: Arc<Mutex<Allocator>>,
		handle: vk::Buffer,
		allocation: Allocation,
		size: u64,
		device_address: u64,
	) -> Self {
		Self {
			inner: Arc::new(Mutex::new(BufferInner {
				device: device.clone(),
				allocator: Some(allocator),
				handle,
				allocation: Some(allocation),
				size,
				device_address,
			})),
		}
	}

	pub fn new_unowned(
		device: &ash::Device,
		handle: vk::Buffer,
		size: u64,
		device_address: u64,
	) -> Self {
		Self {
			inner: Arc::new(Mutex::new(BufferInner {
				device: device.clone(),
				allocator: None,
				handle,
				allocation: None,
				size,
				device_address,
			})),
		}
	}

	pub fn handle(&self) -> vk::Buffer {
		self.inner.lock().unwrap().handle
	}
}

impl RhiBuffer for AshBuffer {
	fn size(&self) -> u64 {
		self.inner.lock().unwrap().size
	}

	fn device_address(&self) -> u64 {
		self.inner.lock().unwrap().device_address
	}

	fn map(&mut self) -> RhiResult<*mut u8> {
		let inner = self.inner.lock().unwrap();
		if let Some(alloc) = &inner.allocation {
			if let Some(ptr) = alloc.mapped_ptr() {
				return Ok(ptr.as_ptr() as *mut u8);
			}
		}
		Err(RhiError::BackendError(
			"Buffer memory is not host visible or cannot be mapped".into(),
		))
	}

	fn unmap(&mut self) {
		// gpu-allocator persistently maps host visible allocations, no-op needed
	}

	fn flush(&mut self, _offset: u64, _size: u64) -> RhiResult<()> {
		Ok(())
	}
}
