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
use winit::window::{CursorGrabMode, Window, WindowAttributes};

use mochi_rhi::{self as rhi, Device, Surface, Swapchain};
use mochi_except::*;


pub struct Display {
	swapchain: Swapchain,
	surface: Surface,
	window: Arc<Window>,
	width: u32,
	height: u32,
	cursor_captured: std::sync::atomic::AtomicBool,
}

impl Display {
	pub fn new(
		event_loop: &ActiveEventLoop,
		device: &Device,
		title: &str,
		width: u32,
		height: u32,
	) -> Result<Self, String> {
		let attributes = WindowAttributes::default()
			.with_title(title)
			.with_inner_size(PhysicalSize::new(width, height));

		let window = Arc::new(
			event_loop
				.create_window(attributes)
				.map_err(|e| format!("Failed to create window: {e}"))?,
		);

		let surface = device.create_surface(&*window)?;
		let swapchain = Swapchain::new(device, &surface, width, height);

		mochi_disp!(
			"Native window created ({}x{}) with swapchain ({} images)",
			width,
			height,
			swapchain.image_count()
		);

		let display = Self {
			swapchain,
			surface,
			window,
			width,
			height,
			cursor_captured: std::sync::atomic::AtomicBool::new(false),
		};

		display.set_cursor_captured(true);

		Ok(display)
	}

	pub fn is_cursor_captured(&self) -> bool {
		self
			.cursor_captured
			.load(std::sync::atomic::Ordering::Relaxed)
	}

	pub fn set_cursor_captured(&self, captured: bool) {
		if captured {
			let res = self
				.window
				.set_cursor_grab(CursorGrabMode::Locked)
				.or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Confined));
			if let Err(e) = res {
				mochi_warn!("Could not grab cursor: {e}");
			}
			self.window.set_cursor_visible(false);
		} else {
			let _ = self.window.set_cursor_grab(CursorGrabMode::None);
			self.window.set_cursor_visible(true);
		}
		self
			.cursor_captured
			.store(captured, std::sync::atomic::Ordering::Relaxed);
	}


	pub fn resize(&mut self, width: u32, height: u32) {
		if width == 0 || height == 0 {
			return;
		}

		self.width = width;
		self.height = height;
		self.swapchain.resize(width, height);
	}

	pub fn acquire_next_image(&mut self) -> u32 {
		self.swapchain.acquire_next_image()
	}

	pub fn acquire_next_image_with_semaphore(&mut self, signal_semaphore: &rhi::Semaphore) -> u32 {
		self
			.swapchain
			.acquire_next_image_with_semaphore(Some(signal_semaphore))
	}

	pub fn present(&mut self, image_index: u32) {
		self.swapchain.present(image_index);
	}

	pub fn present_with_semaphore(&mut self, image_index: u32, wait_semaphore: &rhi::Semaphore) {
		self
			.swapchain
			.present_with_semaphore(image_index, Some(wait_semaphore));
	}

	pub fn render_target(&self, index: u32) -> Option<rhi::RenderTarget<'_>> {
		self.swapchain.render_target(index)
	}

	pub fn set_msaa(&mut self, samples: rhi::MsaaSamples) {
		self.swapchain.set_msaa(samples);
	}

	pub fn msaa_samples(&self) -> rhi::MsaaSamples {
		self.swapchain.msaa_samples()
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
