/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ops::{Deref, DerefMut, Range};
use std::sync::Arc;
use bitflags::bitflags;

use crate::error::{RhiError, RhiResult};
use crate::ffi::*;
use crate::manager::allocator::AllocatorInner;


bitflags! {
	pub struct BufferUsage: u32 {
		const TransferSrc              = 0x00000001;
		const TransferDst              = 0x00000002;
		const UniformBuffer            = 0x00000010;
		const StorageBuffer            = 0x00000020;
		const IndexBuffer              = 0x00000040;
		const VertexBuffer             = 0x00000080;
		const IndirectBuffer           = 0x00000100;
		const DeviceAddress            = 0x00020000;
		const SamplerDescriptorBuffer  = 0x00200000;
		const ResourceDescriptorBuffer = 0x00400000;
		const DescriptorHeap           = 0x10000000;
	}
}

pub struct Buffer {
	allocator: Arc<AllocatorInner>,
	handle: MochiBufferHandle,
	owned: bool,
}

unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}

impl Buffer {
	pub fn new(
		allocator: &crate::Allocator,
		size: u64,
		usage: BufferUsage,
		create: crate::AllocationCreate,
		location: crate::AllocationLocation,
	) -> Self {
		allocator.alloc_buffer(size, usage, create, location)
	}

	pub fn from_handle(allocator: Arc<AllocatorInner>, handle: MochiBufferHandle) -> Self {
		Self { allocator, handle, owned: true }
	}

	pub fn from_handle_unowned(allocator: Arc<AllocatorInner>, handle: MochiBufferHandle) -> Self {
		Self { allocator, handle, owned: false }
	}

	pub fn allocator(&self) -> &Arc<AllocatorInner> {
		&self.allocator
	}

	pub fn handle(&self) -> MochiBufferHandle {
		self.handle
	}

	pub fn size(&self) -> u64 {
		unsafe { mochi_rhi_buffer_get_size(self.handle) }
	}

	pub fn address(&self) -> u64 {
		unsafe { mochi_rhi_buffer_get_device_address(self.handle) }
	}

	pub fn flush(&mut self, rng: Range<u64>) {
		unsafe { mochi_rhi_buffer_flush(self.handle, rng.start, rng.end) }
	}

	pub fn map_scoped(&mut self) -> RhiResult<MappedBuffer<'_>> {
		unsafe { mochi_rhi_buffer_map(self.handle); }
		let ptr = unsafe { mochi_rhi_buffer_get_mapped_ptr(self.handle) };
		if ptr.is_null() {
			return Err(RhiError::BackendError("Failed to map buffer memory".into()));
		}
		let sz = self.size() as usize;
		let slice = unsafe { std::slice::from_raw_parts_mut(ptr, sz) };
		Ok(MappedBuffer {
			buffer: self,
			slice,
		})
	}

	pub fn map(&mut self) {
		unsafe { mochi_rhi_buffer_map(self.handle) }
	}

	pub fn unmap(&mut self) {
		unsafe { mochi_rhi_buffer_unmap(self.handle) }
	}

	pub fn mapped(&mut self) -> Option<&mut [u8]> {
		let ptr = unsafe { mochi_rhi_buffer_get_mapped_ptr(self.handle) };
		if ptr.is_null() {
			None
		} else {
			let sz = self.size() as usize;
			Some(unsafe { std::slice::from_raw_parts_mut(ptr, sz) })
		}
	}
}

impl Drop for Buffer {
	fn drop(&mut self) {
		if self.owned && !self.handle.is_null() {
			unsafe {
				mochi_rhi_buffer_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}


pub struct MappedBuffer<'a> {
	buffer: &'a mut Buffer,
	slice: &'a mut [u8],
}

impl<'a> Deref for MappedBuffer<'a> {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		self.slice
	}
}

impl<'a> DerefMut for MappedBuffer<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.slice
	}
}

impl<'a> Drop for MappedBuffer<'a> {
	fn drop(&mut self) {
		unsafe {
			mochi_rhi_buffer_unmap(self.buffer.handle);
		}
	}
}
