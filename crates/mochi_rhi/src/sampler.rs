/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use crate::ffi::*;
use crate::manager::DeviceInner;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SamplerAddressMode {
	Repeat = 0,
	MirroredRepeat = 1,
	ClampToEdge = 2,
	ClampToBorder = 3,
	MirrorClampToEdge = 4,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SamplerFilter {
	Nearest = 0,
	Linear = 1,
	Cubic = 1000015000,
}

pub struct SamplerInner {
	device: Arc<DeviceInner>,
	handle: MochiSampler2Handle,
}

unsafe impl Send for SamplerInner {}
unsafe impl Sync for SamplerInner {}

impl SamplerInner {
	pub fn handle(&self) -> MochiSampler2Handle {
		self.handle
	}

	pub fn device(&self) -> &Arc<DeviceInner> {
		&self.device
	}
}

impl Drop for SamplerInner {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_sampler2_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}

#[derive(Clone)]
pub struct Sampler2 {
	inner: Arc<SamplerInner>,
}

unsafe impl Send for Sampler2 {}
unsafe impl Sync for Sampler2 {}

impl Sampler2 {
	pub fn new(
		allocator: &crate::Allocator,
		address_mode: mochi_math::Extent<3, SamplerAddressMode>,
		mag_filter: SamplerFilter,
		min_filter: SamplerFilter,
	) -> Self {
		allocator.alloc_sampler2(address_mode, mag_filter, min_filter)
	}

	pub fn from_handle(device: Arc<DeviceInner>, handle: MochiSampler2Handle) -> Self {
		Self {
			inner: Arc::new(SamplerInner { device, handle }),
		}
	}

	pub fn inner(&self) -> &Arc<SamplerInner> {
		&self.inner
	}

	pub fn handle(&self) -> MochiSampler2Handle {
		self.inner.handle
	}
}
