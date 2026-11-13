/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;

use crate::manager::device::Device;
use crate::traits::RhiBackend;
use crate::{Buffer, Image2};


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum TransferTime {
	Now = 0,
	Lazy = 1,
}


#[derive(Clone)]
pub struct Uploader<B: RhiBackend> {
	device: Arc<B::Device>,
}

unsafe impl<B: RhiBackend> Send for Uploader<B> {}
unsafe impl<B: RhiBackend> Sync for Uploader<B> {}

impl<B: RhiBackend> Uploader<B> {
	pub fn new(device: &Device<B>) -> Self {
		Self {
			device: device.raw_arc().clone(),
		}
	}

	pub fn device(&self) -> &B::Device {
		&self.device
	}

	pub fn copy_memory_to_buffer(&self, _time: TransferTime, src: &[u8], dst: &mut Buffer<B>) {
		assert!(
			(src.len() as u64) <= dst.size(),
			"Uploader: Source slice ({} bytes) exceeds destination buffer capacity ({} bytes)",
			src.len(),
			dst.size()
		);
		dst.write_bytes(0, src).expect("Failed to write memory to buffer in Uploader");
	}

	pub fn copy_memory_to_image(&self, _time: TransferTime, src: &[u8], dst: &mut Image2<B>) {
		assert!(!src.is_empty(), "Uploader: Cannot copy empty slice to Image2");
		assert!(dst.width() > 0 && dst.height() > 0, "Uploader: Cannot copy to 0-sized Image2 (extent: ({},{}))", dst.width(), dst.height());
		assert!(
			(src.len() as u64) >= (dst.width() as u64 * dst.height() as u64),
			"Uploader: Source slice ({} bytes) is smaller than image dimension pixels ({}x{})",
			src.len(),
			dst.width(),
			dst.height(),
		);
	}
}
