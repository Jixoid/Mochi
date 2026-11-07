/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::marker::PhantomData;
use std::sync::Arc;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::manager::allocator::Allocator;
use crate::manager::swapchain::Surface;
use crate::traits::{RhiBackend, RhiDevice};
use crate::{Buffer, ImageView2, Sampler2};


#[derive(Clone, Debug)]
pub struct Queue {
	family: u32,
}

impl Queue {
	pub fn new(family: u32) -> Self {
		Self { family }
	}

	pub fn family(&self) -> u32 {
		self.family
	}
}

#[derive(Clone, Debug, Default)]
pub struct QueueGroup {
	primary: Vec<Queue>,
	secondary: Vec<Queue>,
}

impl QueueGroup {
	pub fn new() -> Self {
		Self {
			primary: Vec::new(),
			secondary: Vec::new(),
		}
	}

	pub fn add_primary(&mut self, q: Queue) {
		self.primary.push(q);
	}

	pub fn add_secondary(&mut self, q: Queue) {
		self.secondary.push(q);
	}

	pub fn available(&self) -> bool {
		!self.primary.is_empty() || !self.secondary.is_empty()
	}

	pub fn best(&self) -> &Queue {
		if !self.primary.is_empty() {
			self.primary.first().unwrap()
		} else {
			self.secondary.first().unwrap()
		}
	}

	pub fn bests(&self) -> &[Queue] {
		if !self.primary.is_empty() {
			&self.primary
		} else {
			&self.secondary
		}
	}
}


#[derive(Clone)]
pub struct Device<B: RhiBackend> {
	inner: Arc<B::Device>,
	main_q: Queue,
	graphics_q: QueueGroup,
	compute_q: QueueGroup,
	transfer_q: QueueGroup,
	_marker: PhantomData<B>,
}

unsafe impl<B: RhiBackend> Send for Device<B> {}
unsafe impl<B: RhiBackend> Sync for Device<B> {}

impl<B: RhiBackend> Device<B> {
	pub fn new(app_name: &str, major: u16, minor: u16, patch: u16, build: u16) -> Self {
		let dev = B::create_device(app_name, major, minor, patch, build)
			.expect("Failed to initialize RHI device");
		let q_family = dev.main_queue_family();
		let main_q = Queue::new(q_family);

		let mut graphics_q = QueueGroup::new();
		graphics_q.add_primary(Queue::new(q_family));

		Self {
			inner: Arc::new(dev),
			main_q,
			graphics_q,
			compute_q: QueueGroup::new(),
			transfer_q: QueueGroup::new(),
			_marker: PhantomData,
		}
	}

	pub fn raw(&self) -> &B::Device {
		&self.inner
	}

	pub fn raw_arc(&self) -> &Arc<B::Device> {
		&self.inner
	}

	pub fn wait_idle(&self) {
		let _ = self.inner.wait_idle();
	}

	pub fn main_q(&self) -> &Queue {
		&self.main_q
	}

	pub fn graphics_q(&self) -> &QueueGroup {
		&self.graphics_q
	}

	pub fn compute_q(&self) -> &QueueGroup {
		&self.compute_q
	}

	pub fn transfer_q(&self) -> &QueueGroup {
		&self.transfer_q
	}

	pub fn init_descriptor_heap(&mut self, alloc: &Allocator<B>) {
		let _ = self.inner.init_descriptor_heap(alloc.raw());
	}

	pub fn fini_descriptor_heap(&mut self) {
		self.inner.fini_descriptor_heap();
	}

	pub fn write_texture_descriptor(&mut self, view: &ImageView2<'_, B>, sampler: &Sampler2<B>) -> u32 {
		self.inner.write_texture_descriptor(view.raw(), sampler.raw())
	}

	pub fn descriptor_heap(&self) -> Option<Buffer<B>> {
		self.inner.descriptor_heap().map(Buffer::from_raw)
	}

	pub fn sampler_heap(&self) -> Option<Buffer<B>> {
		self.inner.sampler_heap().map(Buffer::from_raw)
	}

	pub fn create_surface<W: HasWindowHandle + HasDisplayHandle>(&self, window: &W) -> Result<Surface<B>, String> {
		let handle = self.inner.create_surface(window)
			.map_err(|e| format!("Failed to create surface: {e}"))?;
		Ok(Surface::new(self.inner.clone(), handle))
	}
}
