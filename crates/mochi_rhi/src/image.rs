/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use bitflags::bitflags;

use crate::format::Format;
use crate::manager::allocator::{AllocationCreate, AllocationLocation, Allocator};
use crate::traits::RhiBackend;
use crate::types::MsaaSamples;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ImageTiling {
	Optimal = 0,
	Linear = 1,
}

bitflags! {
	#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Image2Desc {
	pub width: u32,
	pub height: u32,
	pub format: Format,
	pub usage: ImageUsage,
	pub tiling: ImageTiling,
	pub create: AllocationCreate,
	pub location: AllocationLocation,
	pub samples: MsaaSamples,
}

impl Image2Desc {
	pub fn new(width: u32, height: u32, format: Format) -> Self {
		Self {
			width,
			height,
			format,
			usage: ImageUsage::empty(),
			tiling: ImageTiling::Optimal,
			create: AllocationCreate::Dedicated,
			location: AllocationLocation::PreferDevice,
			samples: MsaaSamples::Off,
		}
	}
}

pub struct Image2Builder {
	desc: Image2Desc,
}

impl Image2Builder {
	fn new(width: u32, height: u32, format: Format) -> Self {
		Self {
			desc: Image2Desc::new(width, height, format),
		}
	}

	pub fn usage(mut self, usage: ImageUsage) -> Self {
		self.desc.usage = usage;
		self
	}

	pub fn tiling(mut self, tiling: ImageTiling) -> Self {
		self.desc.tiling = tiling;
		self
	}

	pub fn create(mut self, create: AllocationCreate) -> Self {
		self.desc.create = create;
		self
	}

	pub fn location(mut self, location: AllocationLocation) -> Self {
		self.desc.location = location;
		self
	}

	pub fn samples(mut self, samples: MsaaSamples) -> Self {
		self.desc.samples = samples;
		self
	}

	pub fn desc(&self) -> &Image2Desc {
		&self.desc
	}

	pub fn build<B: RhiBackend>(self, allocator: &Allocator<B>) -> Image2<B> {
		allocator.alloc_image2(&self.desc)
	}
}


#[derive(Clone)]
pub struct ImageView2<B: RhiBackend> {
	inner: B::ImageView2,
}

unsafe impl<B: RhiBackend> Send for ImageView2<B> {}
unsafe impl<B: RhiBackend> Sync for ImageView2<B> {}

impl<B: RhiBackend> ImageView2<B> {
	pub fn from_raw(inner: B::ImageView2) -> Self {
		Self { inner }
	}

	pub fn raw(&self) -> &B::ImageView2 {
		&self.inner
	}
}


pub struct Image2<B: RhiBackend> {
	inner: B::Image2,
	width: u32,
	height: u32,
}

unsafe impl<B: RhiBackend> Send for Image2<B> {}
unsafe impl<B: RhiBackend> Sync for Image2<B> {}

impl<B: RhiBackend> Image2<B> {
	pub fn builder(width: u32, height: u32, format: Format) -> Image2Builder {
		Image2Builder::new(width, height, format)
	}

	pub fn from_desc(allocator: &Allocator<B>, desc: &Image2Desc) -> Self {
		allocator.alloc_image2(desc)
	}

	pub fn from_raw(inner: B::Image2, width: u32, height: u32) -> Self {
		Self {
			inner,
			width,
			height,
		}
	}

	pub fn raw(&self) -> &B::Image2 {
		&self.inner
	}

	pub fn width(&self) -> u32 {
		self.width
	}

	pub fn height(&self) -> u32 {
		self.height
	}
}
