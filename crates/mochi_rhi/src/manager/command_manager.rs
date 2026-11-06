/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};

use crate::command::Command;
use crate::ffi::*;
use crate::manager::synchronizer::{Fence, Semaphore};
use crate::manager::DeviceInner;
use crate::Device;


pub struct CommandManagerInner {
	device: Arc<DeviceInner>,
	handle: MochiCommandManagerHandle,
	submit_lock: Mutex<()>,
}

unsafe impl Send for CommandManagerInner {}
unsafe impl Sync for CommandManagerInner {}

impl CommandManagerInner {
	pub fn handle(&self) -> MochiCommandManagerHandle {
		self.handle
	}

	pub fn device(&self) -> &Arc<DeviceInner> {
		&self.device
	}
}

impl Drop for CommandManagerInner {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_command_manager_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}


#[derive(Clone)]
pub struct CommandManager {
	inner: Arc<CommandManagerInner>,
}

unsafe impl Send for CommandManager {}
unsafe impl Sync for CommandManager {}

impl CommandManager {
	pub fn new(device: &Device) -> Self {
		let handle = unsafe { mochi_rhi_command_manager_new(device.handle()) };
		assert!(!handle.is_null(), "RHI driver failed to create CommandManager");
		Self {
			inner: Arc::new(CommandManagerInner {
				device: device.inner().clone(),
				handle,
				submit_lock: Mutex::new(()),
			}),
		}
	}

	pub fn inner(&self) -> &Arc<CommandManagerInner> {
		&self.inner
	}

	pub fn handle(&self) -> MochiCommandManagerHandle {
		self.inner.handle
	}

	pub fn allocate_graphic_command(&self) -> Result<Command, String> {
		let cmd = unsafe { mochi_rhi_command_manager_allocate_graphics(self.inner.handle) };
		if cmd.is_null() {
			Err("RHI driver failed to allocate GraphicCommand buffer".to_string())
		} else {
			Ok(Command::from_handle(self.inner.clone(), cmd))
		}
	}

	pub fn submit(&self, cmd: &Command) {
		let _lock = self.inner.submit_lock.lock().unwrap();
		unsafe { mochi_rhi_command_manager_submit(self.inner.handle, cmd.handle()); }
	}

	pub fn submit_with_sync(
		&self,
		cmd: &Command,
		wait_semaphore: Option<&Semaphore>,
		signal_semaphore: Option<&Semaphore>,
		fence: Option<&Fence>,
	) {
		let wait = wait_semaphore.map(|s| s.0).unwrap_or(std::ptr::null_mut());
		let sig = signal_semaphore.map(|s| s.0).unwrap_or(std::ptr::null_mut());
		let f = fence.map(|f| f.0).unwrap_or(std::ptr::null_mut());
		let _lock = self.inner.submit_lock.lock().unwrap();
		unsafe {
			mochi_rhi_command_manager_submit_with_sync(
				self.inner.handle,
				cmd.handle(),
				wait,
				sig,
				f,
			);
		}
	}
}
