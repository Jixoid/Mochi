/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::marker::PhantomData;
use bitflags::bitflags;
use mochi_math::Extent;

use crate::format::Format;
use crate::manager::allocator::{AllocationCreate, AllocationLocation, Allocator};
use crate::traits::RhiBackend;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ImageTiling {
	Optimal = 0,
	Linear = 1,
}

bitflags! {
	pub struct ImageUsage: u32 {
		const TransferSrc                   = 0x00000001;
		const TransferDst                   = 0x00000002;
		const Sampled                       = 0x00000004;
		const Storage                       = 0x00000008;
		const ColorAttachment               = 0x00000010;
		const DepthStencilAttachment        = 0x00000020;
		const TransientAttachment           = 0x00000040;
		const InputAttachment               = 0x00000080;
		const HostTransfer                  = 0x00400000;
		const VideoDecodeDst                = 0x00000400;
		const VideoDecodeSrc                = 0x00000800;
		const VideoDecodeDpb                = 0x00001000;
		const FragmentDensityMap            = 0x00000200;
		const FragmentShadingRateAttachment = 0x00000100;
		const VideoEncodeDst                = 0x00002000;
		const VideoEncodeSrc                = 0x00004000;
		const VideoEncodeDpb                = 0x00008000;
		const AttachmentFeedbackLoop        = 0x00080000;
	}
}


pub struct ImageView2<'a, B: RhiBackend> {
	inner: &'a B::ImageView,
	_marker: PhantomData<&'a B::Image>,
}

unsafe impl<'a, B: RhiBackend> Send for ImageView2<'a, B> {}
unsafe impl<'a, B: RhiBackend> Sync for ImageView2<'a, B> {}

impl<'a, B: RhiBackend> ImageView2<'a, B> {
	pub fn new(inner: &'a B::ImageView) -> Self {
		Self {
			inner,
			_marker: PhantomData,
		}
	}

	pub fn raw(&self) -> &B::ImageView {
		self.inner
	}
}


pub struct Image2<B: RhiBackend> {
	inner: B::Image,
	ext: Extent<2, u32>,
}

unsafe impl<B: RhiBackend> Send for Image2<B> {}
unsafe impl<B: RhiBackend> Sync for Image2<B> {}

impl<B: RhiBackend> Image2<B> {
	pub fn new(
		allocator: &Allocator<B>,
		ext: Extent<2, u32>,
		format: Format,
		usage: ImageUsage,
		tiling: ImageTiling,
		create: AllocationCreate,
		location: AllocationLocation,
	) -> Self {
		allocator.alloc_image2(ext, format, usage, tiling, create, location)
	}

	pub fn from_raw(inner: B::Image, ext: Extent<2, u32>) -> Self {
		Self { inner, ext }
	}

	pub fn raw(&self) -> &B::Image {
		&self.inner
	}

	pub fn ext(&self) -> Extent<2, u32> {
		self.ext
	}
}
