/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use crate::rhi::{self as rhi, Device, Surface, Swapchain};
use mochi_except::*;


pub struct Display {
	swapchain: Swapchain,
	surface: Surface,
	window: Arc<Window>,
	width: u32,
	height: u32,
}

impl Display {
	pub fn new(event_loop: &ActiveEventLoop, device: &Device, title: &str, width: u32, height: u32) -> Result<Self, String> {
		let attributes = WindowAttributes::default()
			.with_title(title)
			.with_inner_size(PhysicalSize::new(width, height));

		let window = Arc::new(event_loop
			.create_window(attributes)
			.map_err(|e| format!("Failed to create window: {e}"))?);

		let surface = device.create_surface(&*window)?;
		let swapchain = Swapchain::new(device, &surface, width, height);

		mochi_disp!("Native window created ({}x{}) with Vulkan swapchain ({} images)", width, height, swapchain.image_count());

		Ok(Self {
			swapchain,
			surface,
			window,
			width,
			height,
		})
	}


	pub fn resize(&mut self, width: u32, height: u32) {
		if width == 0 || height == 0 { return }

		self.width = width;
		self.height = height;
		self.swapchain.resize(width, height);
	}

	pub fn acquire_next_image(&mut self) -> u32 {
		self.swapchain.acquire_next_image()
	}

	pub fn acquire_next_image_with_semaphore(&mut self, signal_semaphore: &rhi::Semaphore) -> u32 {
		self.swapchain.acquire_next_image_with_semaphore(Some(signal_semaphore))
	}

	pub fn present(&mut self, image_index: u32) {
		self.swapchain.present(image_index);
	}

	pub fn present_with_semaphore(&mut self, image_index: u32, wait_semaphore: &rhi::Semaphore) {
		self.swapchain.present_with_semaphore(image_index, Some(wait_semaphore));
	}

	pub fn render_target(&self, index: u32) -> Option<rhi::RenderTarget<'_>> {
		self.swapchain.render_target(index)
	}

	pub fn render_finished_semaphore(&self, index: u32) -> rhi::Semaphore {
		self.swapchain.render_finished_semaphore(index)
	}

	pub fn request_redraw(&self) {
		self.window.request_redraw();
	}

	pub fn window(&self) -> &Window {
		&self.window
	}


	pub fn width(&self) -> u32 {
		self.width
	}

	pub fn height(&self) -> u32 {
		self.height
	}

	pub fn swapchain(&self) -> &Swapchain {
		&self.swapchain
	}

	pub fn surface(&self) -> &Surface {
		&self.surface
	}
}
