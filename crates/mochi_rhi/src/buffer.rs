/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ops::Range;
use bitflags::bitflags;

use crate::ffi::*;


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
	handle: MochiBufferHandle,
}

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

	pub fn from_handle(handle: MochiBufferHandle) -> Self {
		Self { handle }
	}

	pub fn handle(&self) -> MochiBufferHandle {
		self.handle
	}

	
	pub fn size(&self) -> u64 {
		unsafe { mochi_rhi_buffer_get_size(self.handle) }
	}

	pub fn map(&mut self) {
		unsafe { mochi_rhi_buffer_map(self.handle) }
	}

	pub fn umap(&mut self) {
		unsafe { mochi_rhi_buffer_unmap(self.handle) }
	}

	pub fn flush(&mut self, rng: Range<u64>) {
		unsafe { mochi_rhi_buffer_flush(self.handle, rng.start, rng.end) }
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

	pub fn address(&self) -> u64 {
		unsafe { mochi_rhi_buffer_get_device_address(self.handle) }
	}
}

impl Drop for Buffer {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_buffer_dis(self.handle);
			}
		}
	}
}
