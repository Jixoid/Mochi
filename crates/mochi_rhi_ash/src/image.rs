/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};
use ash::vk;
use gpu_allocator::vulkan::{Allocation, Allocator};
use mochi_math::Extent;

use mochi_rhi::format::Format;
use mochi_rhi::traits::{RhiImage, RhiImageView};


pub struct ImageInner {
	device: ash::Device,
	allocator: Option<Arc<Mutex<Allocator>>>,
	handle: vk::Image,
	allocation: Option<Allocation>,
	ext: Extent<2, u32>,
	format: Format,
}

unsafe impl Send for ImageInner {}
unsafe impl Sync for ImageInner {}

impl Drop for ImageInner {
	fn drop(&mut self) {
		if self.handle != vk::Image::null() {
			unsafe {
				self.device.destroy_image(self.handle, None);
			}
			self.handle = vk::Image::null();
		}
		if let (Some(allocator), Some(alloc)) = (&self.allocator, self.allocation.take()) {
			let mut lock = allocator.lock().unwrap();
			let _ = lock.free(alloc);
		}
	}
}

#[derive(Clone)]
pub struct AshImage {
	inner: Arc<Mutex<ImageInner>>,
}

unsafe impl Send for AshImage {}
unsafe impl Sync for AshImage {}

impl AshImage {
	pub fn new(
		device: &ash::Device,
		allocator: Arc<Mutex<Allocator>>,
		handle: vk::Image,
		allocation: Allocation,
		ext: Extent<2, u32>,
		format: Format,
	) -> Self {
		Self {
			inner: Arc::new(Mutex::new(ImageInner {
				device: device.clone(),
				allocator: Some(allocator),
				handle,
				allocation: Some(allocation),
				ext,
				format,
			})),
		}
	}

	pub fn new_unowned(device: &ash::Device, handle: vk::Image, ext: Extent<2, u32>, format: Format) -> Self {
		Self {
			inner: Arc::new(Mutex::new(ImageInner {
				device: device.clone(),
				allocator: None,
				handle,
				allocation: None,
				ext,
				format,
			})),
		}
	}

	pub fn handle(&self) -> vk::Image {
		self.inner.lock().unwrap().handle
	}
}

impl RhiImage for AshImage {
	fn ext(&self) -> Extent<2, u32> {
		self.inner.lock().unwrap().ext
	}

	fn format(&self) -> Format {
		self.inner.lock().unwrap().format
	}
}


pub struct ImageViewInner {
	device: ash::Device,
	handle: vk::ImageView,
}

unsafe impl Send for ImageViewInner {}
unsafe impl Sync for ImageViewInner {}

impl Drop for ImageViewInner {
	fn drop(&mut self) {
		if self.handle != vk::ImageView::null() {
			unsafe {
				self.device.destroy_image_view(self.handle, None);
			}
			self.handle = vk::ImageView::null();
		}
	}
}

#[derive(Clone)]
pub struct AshImageView {
	inner: Arc<ImageViewInner>,
}

unsafe impl Send for AshImageView {}
unsafe impl Sync for AshImageView {}

impl AshImageView {
	pub fn new(device: &ash::Device, handle: vk::ImageView) -> Self {
		Self {
			inner: Arc::new(ImageViewInner {
				device: device.clone(),
				handle,
			}),
		}
	}

	pub fn handle(&self) -> vk::ImageView {
		self.inner.handle
	}
}

impl RhiImageView for AshImageView {}
