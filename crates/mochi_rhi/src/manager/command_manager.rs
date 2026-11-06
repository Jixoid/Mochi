/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::command::Command;
use crate::ffi::*;
use crate::Device;


pub struct CommandManager {
	handle: MochiCommandManagerHandle,
}

impl CommandManager {
	pub fn new(device: &Device) -> Self {
		let handle = unsafe { mochi_rhi_command_manager_new(device.handle()) };
		assert!(!handle.is_null(), "RHI driver failed to create CommandManager");
		Self { handle }
	}

	pub fn handle(&self) -> MochiCommandManagerHandle {
		self.handle
	}


	pub fn allocate_graphic_command(&self) -> Result<Command, String> {
		let cmd = unsafe { mochi_rhi_command_manager_allocate_graphics(self.handle) };
		if cmd.is_null() {
			Err("RHI driver failed to allocate GraphicCommand buffer".to_string())
		} else {
			Ok(Command::from_handle(cmd))
		}
	}

	
	pub fn submit(&self, cmd: &Command) {
		unsafe { mochi_rhi_command_manager_submit(self.handle, cmd.handle()); }
	}

	pub fn submit_with_sync(&self, cmd: &Command, wait_semaphore: *mut std::ffi::c_void, signal_semaphore: *mut std::ffi::c_void, fence: *mut std::ffi::c_void) {
		unsafe {
			mochi_rhi_command_manager_submit_with_sync(
				self.handle,
				cmd.handle(),
				wait_semaphore,
				signal_semaphore,
				fence,
			);
		}
	}
}

impl Drop for CommandManager {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_command_manager_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}
