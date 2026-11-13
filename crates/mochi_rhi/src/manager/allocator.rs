/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use bitflags::bitflags;

use crate::manager::device::Device;
use crate::traits::{RhiAllocator, RhiBackend};
use crate::{
	Buffer, BufferUsage, Image2, Image2Desc, Sampler2, SamplerAddressMode, SamplerFilter,
};


bitflags! {
	#[derive(Copy, Clone, Debug, PartialEq, Eq)]
	pub struct AllocationCreate: u32 {
		const Dedicated           = 0x00000001;
		const Mapped              = 0x00000004;
		const HostSequentialWrite = 0x00000400;
		const HostRandomAccess    = 0x00000800;
		const HostTransfer        = 0x00001000;
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AllocationLocation {
	Auto = 7,
	PreferDevice = 8,
	PreferHost = 9,
}


#[derive(Clone)]
pub struct Allocator<B: RhiBackend> {
	inner: Arc<B::Allocator>,
}

unsafe impl<B: RhiBackend> Send for Allocator<B> {}
unsafe impl<B: RhiBackend> Sync for Allocator<B> {}

impl<B: RhiBackend> Allocator<B> {
	pub fn new(device: &Device<B>) -> Self {
		let alloc = B::create_allocator(device.raw())
			.expect("Failed to initialize RHI allocator");
		Self {
			inner: Arc::new(alloc),
		}
	}

	pub fn raw(&self) -> &B::Allocator {
		&self.inner
	}

	pub fn alloc_buffer(
		&self,
		size: u64,
		usage: BufferUsage,
		create: AllocationCreate,
		location: AllocationLocation,
	) -> Buffer<B> {
		let buf = self
			.inner
			.alloc_buffer(size, usage, create, location)
			.expect("Buffer allocation failed");
		Buffer::from_raw(buf)
	}

	pub fn alloc_image2(&self, desc: &Image2Desc) -> Image2<B> {
		let img = self
			.inner
			.alloc_image2(
				desc.width,
				desc.height,
				desc.format,
				desc.usage,
				desc.tiling,
				desc.create,
				desc.location,
				desc.samples,
			)
			.expect("Image allocation failed");
		Image2::from_raw(img, desc.width, desc.height)
	}

	pub fn alloc_sampler2(
		&self,
		address_mode: [SamplerAddressMode; 3],
		mag_filter: SamplerFilter,
		min_filter: SamplerFilter,
	) -> Sampler2<B> {
		let samp = self
			.inner
			.alloc_sampler(address_mode, mag_filter, min_filter)
			.expect("Sampler allocation failed");
		Sampler2::from_raw(samp)
	}

	pub fn support_statistics(&self) -> bool {
		true
	}

	pub fn stat_total_vram(&self) -> u64 {
		0
	}

	pub fn stat_cache_vram(&self) -> u64 {
		0
	}

	pub fn stat_used_vram(&self) -> u64 {
		0
	}

	pub fn stat_live_buffer(&self) -> u64 {
		0
	}

	pub fn stat_live_image2(&self) -> u64 {
		0
	}
}
