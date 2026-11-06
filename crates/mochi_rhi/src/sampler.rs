/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::ffi::*;


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
	Cubic = 1000015000, // requires extension
}


pub struct Sampler2 {
	handle: MochiSampler2Handle,
}

impl Sampler2 {
	pub fn new(
		allocator: &crate::Allocator,
		address_mode: mochi_math::Extent<3, SamplerAddressMode>,
		mag_filter: SamplerFilter,
		min_filter: SamplerFilter,
	) -> Self {
		allocator.alloc_sampler2(address_mode, mag_filter, min_filter)
	}

	pub fn from_handle(handle: MochiSampler2Handle) -> Self {
		Self { handle }
	}

	pub fn handle(&self) -> MochiSampler2Handle {
		self.handle
	}
}

impl Drop for Sampler2 {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_sampler2_dis(self.handle);
			}
		}
	}
}
