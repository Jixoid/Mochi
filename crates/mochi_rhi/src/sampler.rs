/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_math::Extent;

use crate::manager::allocator::Allocator;
use crate::traits::RhiBackend;


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


#[derive(Clone)]
pub struct Sampler2<B: RhiBackend> {
	inner: B::Sampler,
}

unsafe impl<B: RhiBackend> Send for Sampler2<B> {}
unsafe impl<B: RhiBackend> Sync for Sampler2<B> {}

impl<B: RhiBackend> Sampler2<B> {
	pub fn new(
		allocator: &Allocator<B>,
		address_mode: Extent<3, SamplerAddressMode>,
		mag_filter: SamplerFilter,
		min_filter: SamplerFilter,
	) -> Self {
		allocator.alloc_sampler2(address_mode, mag_filter, min_filter)
	}

	pub fn from_raw(inner: B::Sampler) -> Self {
		Self { inner }
	}

	pub fn raw(&self) -> &B::Sampler {
		&self.inner
	}
}
