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

use mochi_rhi::format::Format;
use mochi_rhi::traits::{RhiImage2, RhiImageView2};


pub struct Image2Inner {
	device: ash::Device,
	allocator: Option<Arc<Mutex<Allocator>>>,
	handle: vk::Image,
	allocation: Option<Allocation>,
	width: u32,
	height: u32,
	format: Format,
}

unsafe impl Send for Image2Inner {}
unsafe impl Sync for Image2Inner {}

impl Drop for Image2Inner {
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
pub struct AshImage2 {
	inner: Arc<Mutex<Image2Inner>>,
}

unsafe impl Send for AshImage2 {}
unsafe impl Sync for AshImage2 {}

impl AshImage2 {
	pub fn new(
		device: &ash::Device,
		allocator: Arc<Mutex<Allocator>>,
		handle: vk::Image,
		allocation: Allocation,
		width: u32,
		height: u32,
		format: Format,
	) -> Self {
		Self {
			inner: Arc::new(Mutex::new(Image2Inner {
				device: device.clone(),
				allocator: Some(allocator),
				handle,
				allocation: Some(allocation),
				width,
				height,
				format,
			})),
		}
	}

	pub fn new_unowned(
		device: &ash::Device,
		handle: vk::Image,
		width: u32,
		height: u32,
		format: Format,
	) -> Self {
		Self {
			inner: Arc::new(Mutex::new(Image2Inner {
				device: device.clone(),
				allocator: None,
				handle,
				allocation: None,
				width,
				height,
				format,
			})),
		}
	}

	pub fn handle(&self) -> vk::Image {
		self.inner.lock().unwrap().handle
	}
}

impl RhiImage2 for AshImage2 {
	fn width(&self) -> u32 {
		self.inner.lock().unwrap().width
	}

	fn height(&self) -> u32 {
		self.inner.lock().unwrap().height
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
pub struct AshImageView2 {
	inner: Arc<ImageViewInner>,
}

unsafe impl Send for AshImageView2 {}
unsafe impl Sync for AshImageView2 {}

impl AshImageView2 {
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

impl RhiImageView2 for AshImageView2 {}
