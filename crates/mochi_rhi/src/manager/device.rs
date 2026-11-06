/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ffi::CString;
use std::sync::{Arc, Mutex};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};

use crate::ffi::*;
use crate::manager::allocator::AllocatorInner;
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


pub struct DeviceInner {
	handle: MochiDeviceHandle,
	descriptor_allocator: Mutex<Option<Arc<AllocatorInner>>>,
}

unsafe impl Send for DeviceInner {}
unsafe impl Sync for DeviceInner {}

impl DeviceInner {
	pub fn handle(&self) -> MochiDeviceHandle {
		self.handle
	}
}

impl Drop for DeviceInner {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_device_wait_idle(self.handle);
				mochi_rhi_device_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}


#[derive(Clone)]
pub struct Device {
	inner: Arc<DeviceInner>,
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
			inner: Arc::new(DeviceInner {
				handle,
				descriptor_allocator: Mutex::new(None),
			}),
			main_q,
			graphics_q,
			compute_q: QueueGroup::new(),
			transfer_q: QueueGroup::new(),
		}
	}

	pub fn inner(&self) -> &Arc<DeviceInner> {
		&self.inner
	}

	pub fn handle(&self) -> MochiDeviceHandle {
		self.inner.handle
	}

	pub fn wait_idle(&self) {
		unsafe { mochi_rhi_device_wait_idle(self.inner.handle); }
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

	pub fn init_descriptor_heap(&mut self, alloc: &Allocator) {
		*self.inner.descriptor_allocator.lock().unwrap() = Some(alloc.inner().clone());
		unsafe { mochi_rhi_device_init_descriptor_heap(self.inner.handle, alloc.handle()); }
	}

	pub fn fini_descriptor_heap(&mut self) {
		*self.inner.descriptor_allocator.lock().unwrap() = None;
		unsafe { mochi_rhi_device_fini_descriptor_heap(self.inner.handle); }
	}

	pub fn write_texture_descriptor(&mut self, view: &ImageView2<'_>, sampler: &Sampler2) -> u32 {
		unsafe { mochi_rhi_device_write_texture_descriptor(self.inner.handle, view.handle(), sampler.handle()) }
	}

	pub fn descriptor_heap(&self) -> Option<Buffer> {
		let h = unsafe { mochi_rhi_device_get_descriptor_heap(self.inner.handle) };
		if h.is_null() {
			None
		} else {
			let alloc_opt = self.inner.descriptor_allocator.lock().unwrap().clone();
			alloc_opt.map(|alloc| Buffer::from_handle_unowned(alloc, h))
		}
	}

	pub fn sampler_heap(&self) -> Option<Buffer> {
		let h = unsafe { mochi_rhi_device_get_sampler_heap(self.inner.handle) };
		if h.is_null() {
			None
		} else {
			let alloc_opt = self.inner.descriptor_allocator.lock().unwrap().clone();
			alloc_opt.map(|alloc| Buffer::from_handle_unowned(alloc, h))
		}
	}

	pub fn create_surface<W: HasWindowHandle + HasDisplayHandle>(&self, window: &W) -> Result<Surface, String> {
		let display_handle = window
			.display_handle()
			.map_err(|e| format!("failed to get display handle: {e}"))?;

		let window_handle = window
			.window_handle()
			.map_err(|e| format!("failed to get window handle: {e}"))?;

		let dev_handle = self.inner.handle;
		let surface_handle = match (display_handle.as_raw(), window_handle.as_raw()) {
			(RawDisplayHandle::Wayland(d), RawWindowHandle::Wayland(w)) => unsafe {
				mochi_rhi_create_surface_wayland(
					dev_handle,
					d.display.as_ptr(),
					w.surface.as_ptr(),
				)
			},
			(RawDisplayHandle::Xlib(d), RawWindowHandle::Xlib(w)) => {
				let dpy = d.display.map(|p| p.as_ptr()).unwrap_or(std::ptr::null_mut());
				unsafe { mochi_rhi_create_surface_xlib(dev_handle, dpy, w.window) }
			}
			(RawDisplayHandle::Xcb(d), RawWindowHandle::Xcb(w)) => {
				let conn = d.connection.map(|p| p.as_ptr()).unwrap_or(std::ptr::null_mut());
				unsafe { mochi_rhi_create_surface_xcb(dev_handle, conn, w.window.get()) }
			}
			(RawDisplayHandle::Windows(_), RawWindowHandle::Win32(w)) => {
				let hinstance = w
					.hinstance
					.map(|h| h.get() as *mut std::ffi::c_void)
					.unwrap_or(std::ptr::null_mut());
				let hwnd = w.hwnd.get() as *mut std::ffi::c_void;
				unsafe { mochi_rhi_create_surface_win32(dev_handle, hinstance, hwnd) }
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

		Ok(Surface::new(self.inner.clone(), surface_handle))
	}
}
