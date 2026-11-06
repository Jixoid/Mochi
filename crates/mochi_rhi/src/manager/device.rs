/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ffi::CString;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};

use crate::ffi::*;
use crate::{Allocator, Buffer, ImageView2, Sampler2, Surface};


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
	primary: Vec<Queue>,   // Dedicated (specialized) queues
	secondary: Vec<Queue>, // Shared queues
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


pub struct Device {
	handle: MochiDeviceHandle,
	main_q: Queue,
	graphics_q: QueueGroup,
	compute_q: QueueGroup,
	transfer_q: QueueGroup,
}

impl Device {
	pub fn new(app_name: &str, major: u16, minor: u16, patch: u16, build: u16) -> Self {
		let c_str = CString::new(app_name).expect("c string failed");
		let handle = unsafe { mochi_rhi_device_new(c_str.as_ptr(), major, minor, patch, build) };
		assert!(!handle.is_null(), "RHI driver failed to create Device");

		let q_family = unsafe { mochi_rhi_device_get_main_queue_family(handle) };
		let main_q = Queue::new(q_family);

		let mut graphics_q = QueueGroup::new();
		graphics_q.add_primary(Queue::new(q_family));

		Self {
			handle,
			main_q,
			graphics_q,
			compute_q: QueueGroup::new(),
			transfer_q: QueueGroup::new(),
		}
	}

	pub fn handle(&self) -> MochiDeviceHandle {
		self.handle
	}


	pub fn wait_idle(&self) {
		unsafe { mochi_rhi_device_wait_idle(self.handle); }
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


	pub fn init_descriptor_heap(&self, alloc: &Allocator) {
		unsafe { mochi_rhi_device_init_descriptor_heap(self.handle, alloc.handle()); }
	}

	pub fn fini_descriptor_heap(&self) {
		unsafe { mochi_rhi_device_fini_descriptor_heap(self.handle); }
	}


	pub fn write_texture_descriptor(&self, view: &ImageView2, sampler: &Sampler2) -> u32 {
		unsafe { mochi_rhi_device_write_texture_descriptor(self.handle, view.handle(), sampler.handle()) }
	}

	pub fn descriptor_heap(&self) -> Option<Buffer> {
		let h = unsafe { mochi_rhi_device_get_descriptor_heap(self.handle) };
		if h.is_null() {
			None
		} else {
			Some(Buffer::from_handle(h))
		}
	}

	pub fn sampler_heap(&self) -> Option<Buffer> {
		let h = unsafe { mochi_rhi_device_get_sampler_heap(self.handle) };
		if h.is_null() {
			None
		} else {
			Some(Buffer::from_handle(h))
		}
	}


	pub fn create_surface<W: HasWindowHandle + HasDisplayHandle>(&self, window: &W) -> Result<Surface, String> {
		let display_handle = window
			.display_handle()
			.map_err(|e| format!("failed to get display handle: {e}"))?;

		let window_handle = window
			.window_handle()
			.map_err(|e| format!("failed to get window handle: {e}"))?;

		let surface_handle = match (display_handle.as_raw(), window_handle.as_raw()) {
			(RawDisplayHandle::Wayland(d), RawWindowHandle::Wayland(w)) => unsafe {
				mochi_rhi_create_surface_wayland(
					self.handle,
					d.display.as_ptr(),
					w.surface.as_ptr(),
				)
			},
			(RawDisplayHandle::Xlib(d), RawWindowHandle::Xlib(w)) => {
				let dpy = d.display.map(|p| p.as_ptr()).unwrap_or(std::ptr::null_mut());
				unsafe { mochi_rhi_create_surface_xlib(self.handle, dpy, w.window) }
			}
			(RawDisplayHandle::Xcb(d), RawWindowHandle::Xcb(w)) => {
				let conn = d.connection.map(|p| p.as_ptr()).unwrap_or(std::ptr::null_mut());
				unsafe { mochi_rhi_create_surface_xcb(self.handle, conn, w.window.get()) }
			}
			(RawDisplayHandle::Windows(_), RawWindowHandle::Win32(w)) => {
				let hinstance = w
					.hinstance
					.map(|h| h.get() as *mut std::ffi::c_void)
					.unwrap_or(std::ptr::null_mut());
				let hwnd = w.hwnd.get() as *mut std::ffi::c_void;
				unsafe { mochi_rhi_create_surface_win32(self.handle, hinstance, hwnd) }
			}
			(disp, win) => {
				return Err(format!(
					"unsupported window/display handle combination: {:?}, {:?}",
					disp, win
				));
			}
		};

		if surface_handle == 0 {
			return Err("failed to create surface".to_string());
		}

		Ok(Surface::new(self.handle, surface_handle))
	}

}

impl Drop for Device {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_device_wait_idle(self.handle);
				mochi_rhi_device_dis(self.handle);
			}
		}
	}
}
