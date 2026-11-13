/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ops::{Deref, DerefMut, Range};
use bitflags::bitflags;

use crate::error::RhiResult;
use crate::manager::allocator::{AllocationCreate, AllocationLocation, Allocator};
use crate::traits::{RhiBackend, RhiBuffer};


bitflags! {
	#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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


pub struct Buffer<B: RhiBackend> {
	inner: B::Buffer,
}

unsafe impl<B: RhiBackend> Send for Buffer<B> {}
unsafe impl<B: RhiBackend> Sync for Buffer<B> {}

impl<B: RhiBackend> Buffer<B> {
	pub fn new(
		allocator: &Allocator<B>,
		size: u64,
		usage: BufferUsage,
		create: AllocationCreate,
		location: AllocationLocation,
	) -> Self {
		allocator.alloc_buffer(size, usage, create, location)
	}

	pub fn from_slice<T: Copy>(
		allocator: &Allocator<B>,
		usage: BufferUsage,
		data: &[T],
	) -> Result<Self, crate::error::RhiError> {
		let byte_size = (std::mem::size_of::<T>() * data.len()) as u64;
		let mut buf = allocator.alloc_buffer(
			byte_size,
			usage,
			AllocationCreate::Mapped,
			AllocationLocation::Auto,
		);
		let slice = unsafe {
			std::slice::from_raw_parts(data.as_ptr() as *const u8, byte_size as usize)
		};
		buf.write_bytes(0, slice)?;
		Ok(buf)
	}

	pub fn from_raw(inner: B::Buffer) -> Self {
		Self { inner }
	}

	pub fn raw(&self) -> &B::Buffer {
		&self.inner
	}

	pub fn raw_mut(&mut self) -> &mut B::Buffer {
		&mut self.inner
	}

	pub fn size(&self) -> u64 {
		self.inner.size()
	}

	pub fn address(&self) -> u64 {
		self.inner.device_address()
	}

	pub fn flush(&mut self, rng: Range<u64>) {
		let _ = self
			.inner
			.flush(rng.start, rng.end.saturating_sub(rng.start));
	}

	pub fn map(&mut self) {
		let _ = self.inner.map();
	}

	pub fn unmap(&mut self) {
		self.inner.unmap();
	}

	pub fn write_bytes(&mut self, offset: usize, data: &[u8]) -> RhiResult<()> {
		let ptr = self.inner.map()?;
		unsafe {
			let dst = ptr.add(offset);
			std::ptr::copy_nonoverlapping(data.as_ptr(), dst, data.len());
		}
		self.inner.unmap();
		Ok(())
	}

	pub fn map_scoped(&mut self) -> RhiResult<MappedBuffer<'_, B>> {
		let ptr = self.inner.map()?;
		let sz = self.inner.size() as usize;
		let slice = unsafe { std::slice::from_raw_parts_mut(ptr, sz) };
		Ok(MappedBuffer {
			buffer: self,
			slice,
		})
	}

	pub fn mapped(&mut self) -> Option<&mut [u8]> {
		if let Ok(ptr) = self.inner.map() {
			let sz = self.inner.size() as usize;
			Some(unsafe { std::slice::from_raw_parts_mut(ptr, sz) })
		} else {
			None
		}
	}
}


pub struct MappedBuffer<'a, B: RhiBackend> {
	buffer: &'a mut Buffer<B>,
	slice: &'a mut [u8],
}

impl<'a, B: RhiBackend> Deref for MappedBuffer<'a, B> {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		self.slice
	}
}

impl<'a, B: RhiBackend> DerefMut for MappedBuffer<'a, B> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.slice
	}
}

impl<'a, B: RhiBackend> Drop for MappedBuffer<'a, B> {
	fn drop(&mut self) {
		self.buffer.unmap();
	}
}
