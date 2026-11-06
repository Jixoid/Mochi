/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use bitflags::bitflags;
use mochi_math::Extent;

use crate::{Device, ffi::*};
use crate::manager::DeviceInner;
use crate::{Buffer, BufferUsage, Format, Image2, ImageTiling, ImageUsage, Sampler2, SamplerAddressMode, SamplerFilter};


bitflags! {
	pub struct AllocationCreate: u32 {
		const Dedicated           = 0x00000001;
		const Mapped              = 0x00000004;
		const HostSequentialWrite = 0x00000400;
		const HostRandomAccess    = 0x00000800;
		const HostTransfer        = 0x00001000;
	}
}

pub enum AllocationLocation {
	Auto = 7,
	PreferDevice = 8,
	PreferHost = 9,
}


pub struct AllocatorInner {
	device: Arc<DeviceInner>,
	handle: MochiAllocatorHandle,
}

unsafe impl Send for AllocatorInner {}
unsafe impl Sync for AllocatorInner {}

impl AllocatorInner {
	pub fn handle(&self) -> MochiAllocatorHandle {
		self.handle
	}

	pub fn device(&self) -> &Arc<DeviceInner> {
		&self.device
	}
}

impl Drop for AllocatorInner {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_allocator_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}


#[derive(Clone)]
pub struct Allocator {
	inner: Arc<AllocatorInner>,
}

unsafe impl Send for Allocator {}
unsafe impl Sync for Allocator {}

impl Allocator {
	pub fn new(device: &Device) -> Self {
		let handle = unsafe { mochi_rhi_allocator_new(device.handle()) };
		assert!(!handle.is_null(), "RHI driver failed to create Allocator");
		Self {
			inner: Arc::new(AllocatorInner {
				device: device.inner().clone(),
				handle,
			}),
		}
	}

	pub fn inner(&self) -> &Arc<AllocatorInner> {
		&self.inner
	}

	pub fn handle(&self) -> MochiAllocatorHandle {
		self.inner.handle
	}


	pub fn alloc_buffer(
		&self,
		size: u64,
		usage: BufferUsage,
		create: AllocationCreate,
		location: AllocationLocation,
	) -> Buffer {
		let handle = unsafe {
			mochi_rhi_allocator_alloc_buffer(
				self.inner.handle,
				size,
				usage.bits(),
				create.bits(),
				location as u32,
			)
		};
		assert!(!handle.is_null(), "RHI driver failed to allocate Buffer");
		Buffer::from_handle(self.inner.clone(), handle)
	}

	pub fn alloc_image2(
		&self,
		ext: Extent<2, u32>,
		format: Format,
		usage: ImageUsage,
		tiling: ImageTiling,
		create: AllocationCreate,
		location: AllocationLocation,
	) -> Image2 {
		let handle = unsafe {
			mochi_rhi_allocator_alloc_image2(
				self.inner.handle,
				ext[0],
				ext[1],
				format as u32,
				usage.bits(),
				tiling as u32,
				create.bits(),
				location as u32,
			)
		};
		assert!(!handle.is_null(), "RHI driver failed to allocate Image2");
		Image2::from_handle(self.inner.clone(), handle, ext)
	}

	pub fn alloc_sampler2(
		&self,
		address_mode: Extent<3, SamplerAddressMode>,
		mag_filter: SamplerFilter,
		min_filter: SamplerFilter,
	) -> Sampler2 {
		let handle = unsafe {
			mochi_rhi_allocator_alloc_sampler2(
				self.inner.handle,
				address_mode[0] as u32,
				address_mode[1] as u32,
				address_mode[2] as u32,
				mag_filter as u32,
				min_filter as u32,
			)
		};
		assert!(!handle.is_null(), "RHI driver failed to allocate Sampler2");
		Sampler2::from_handle(self.inner.device.clone(), handle)
	}

	
	// Statistics
	pub fn support_statistics(&self) -> bool {
		unsafe { mochi_rhi_allocator_support_statistics(self.inner.handle) }
	}

	pub fn stat_total_vram(&self) -> u64 {
		unsafe { mochi_rhi_allocator_stat_total_vram(self.inner.handle) }
	}

	pub fn stat_cache_vram(&self) -> u64 {
		unsafe { mochi_rhi_allocator_stat_cache_vram(self.inner.handle) }
	}

	pub fn stat_used_vram(&self) -> u64 {
		unsafe { mochi_rhi_allocator_stat_used_vram(self.inner.handle) }
	}

	pub fn stat_live_buffer(&self) -> u64 {
		unsafe { mochi_rhi_allocator_stat_live_buffer(self.inner.handle) }
	}

	pub fn stat_live_image2(&self) -> u64 {
		unsafe { mochi_rhi_allocator_stat_live_image2(self.inner.handle) }
	}
}
