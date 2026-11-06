/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use bitflags::bitflags;
use mochi_math::Extent;

use crate::ffi::*;


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


pub struct ImageView2 {
	handle: MochiImage2Handle,
}

impl ImageView2 {
	pub fn from_handle(handle: MochiImage2Handle) -> Self {
		Self { handle }
	}

	pub fn handle(&self) -> MochiImage2Handle {
		self.handle
	}
}

pub struct Image2 {
	handle: MochiImage2Handle,
	ext: Extent<2, u32>,
}

impl Image2 {
	pub fn new(
		allocator: &crate::Allocator,
		ext: Extent<2, u32>,
		format: crate::Format,
		usage: ImageUsage,
		tiling: ImageTiling,
		create: crate::AllocationCreate,
		location: crate::AllocationLocation,
	) -> Self {
		allocator.alloc_image2(ext, format, usage, tiling, create, location)
	}

	pub fn from_handle(handle: MochiImage2Handle, ext: Extent<2, u32>) -> Self {
		Self { handle, ext }
	}

	pub fn handle(&self) -> MochiImage2Handle {
		self.handle
	}

	pub fn ext(&self) -> Extent<2, u32> {
		self.ext
	}

	pub fn make_view(&self) -> ImageView2 {
		ImageView2::from_handle(self.handle)
	}
}

impl Drop for Image2 {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_image2_dis(self.handle);
			}
		}
	}
}
