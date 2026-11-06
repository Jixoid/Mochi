/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use crate::rhi::{Device, Surface, Swapchain};


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

		println!("[Display] Native window created ({}x{}) with Vulkan swapchain ({} images)",width,height,swapchain.image_count());

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

	
	pub fn acquire_next_image(&self) -> u32 {
		self.swapchain.acquire_next_image()
	}

	pub fn acquire_next_image_with_semaphore(&self, signal_semaphore: *mut std::ffi::c_void) -> u32 {
		self.swapchain.acquire_next_image_with_semaphore(signal_semaphore)
	}

	pub fn present(&self, image_index: u32) {
		self.swapchain.present(image_index);
	}

	pub fn present_with_semaphore(&self, image_index: u32, wait_semaphore: *mut std::ffi::c_void) {
		self.swapchain.present_with_semaphore(image_index, wait_semaphore);
	}

	pub fn render_target(&self, index: u32) -> Option<mochi_rhi::RenderTarget> {
		self.swapchain.render_target(index)
	}

	pub fn render_finished_semaphore(&self, index: u32) -> *mut std::ffi::c_void {
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
