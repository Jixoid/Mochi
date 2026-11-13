/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};

use crate::manager::device::Device;
use crate::render_target::RenderTarget;
use crate::traits::{RhiBackend, RhiDevice, RhiSwapchain};
use crate::types::MsaaSamples;


pub struct Surface<B: RhiBackend> {
	device: Arc<B::Device>,
	handle: u64,
}

unsafe impl<B: RhiBackend> Send for Surface<B> {}
unsafe impl<B: RhiBackend> Sync for Surface<B> {}

impl<B: RhiBackend> Surface<B> {
	pub fn new(device: Arc<B::Device>, handle: u64) -> Self {
		Self { device, handle }
	}

	pub fn handle(&self) -> u64 {
		self.handle
	}
}

impl<B: RhiBackend> Drop for Surface<B> {
	fn drop(&mut self) {
		if self.handle != 0 {
			self.device.destroy_surface(self.handle);
		}
	}
}


#[derive(Clone)]
pub struct Swapchain<B: RhiBackend> {
	inner: Arc<Mutex<B::Swapchain>>,
}

unsafe impl<B: RhiBackend> Send for Swapchain<B> {}
unsafe impl<B: RhiBackend> Sync for Swapchain<B> {}

impl<B: RhiBackend> Swapchain<B> {
	pub fn new(device: &Device<B>, surface: &Surface<B>, width: u32, height: u32) -> Self {
		let sw = B::create_swapchain(device.raw(), surface.handle(), width, height)
			.expect("Failed to create swapchain");
		Self {
			inner: Arc::new(Mutex::new(sw)),
		}
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		self
			.inner
			.lock()
			.unwrap()
			.resize(width, height)
			.expect("Failed to resize swapchain");
	}

	pub fn set_msaa(&mut self, samples: MsaaSamples) {
		self
			.inner
			.lock()
			.unwrap()
			.set_msaa(samples)
			.expect("Failed to set swapchain MSAA");
	}

	pub fn msaa_samples(&self) -> MsaaSamples {
		self.inner.lock().unwrap().msaa_samples()
	}

	pub fn acquire_next_image(&mut self) -> u32 {
		self.acquire_next_image_with_semaphore(None)
	}

	pub fn acquire_next_image_with_semaphore(
		&mut self,
		signal_semaphore: Option<&B::Semaphore>,
	) -> u32 {
		self
			.inner
			.lock()
			.unwrap()
			.acquire_next_image(signal_semaphore)
			.expect("Failed to acquire next swapchain image")
	}

	pub fn present(&mut self, image_index: u32) {
		self.present_with_semaphore(image_index, None);
	}

	pub fn present_with_semaphore(
		&mut self,
		image_index: u32,
		wait_semaphore: Option<&B::Semaphore>,
	) {
		let _ = self
			.inner
			.lock()
			.unwrap()
			.present(image_index, wait_semaphore);
	}

	pub fn image_count(&self) -> u32 {
		self.inner.lock().unwrap().image_count()
	}

	pub fn render_target<'a>(&'a self, index: u32) -> Option<RenderTarget<'a, B>> {
		let lock = self.inner.lock().unwrap();
		lock
			.render_target(index)
			.cloned()
			.map(RenderTarget::from_raw)
	}

	pub fn render_finished_semaphore(&self, index: u32) -> B::Semaphore {
		self
			.inner
			.lock()
			.unwrap()
			.render_finished_semaphore(index)
			.clone()
	}
}
