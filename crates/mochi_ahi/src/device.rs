/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ffi::CString;

use crate::ffi::*;


pub struct Device {
	handle: MochiAudioDeviceHandle,
}

impl Device {
	pub fn new(app_name: &str, major: u16, minor: u16, patch: u16, build: u16) -> Self {
		let c_str = CString::new(app_name).expect("CString::new failed");
		let handle = unsafe { mochi_ahi_device_new(c_str.as_ptr(), major, minor, patch, build) };

		assert!(!handle.is_null(), "C++ Miniaudio driver failed to create audio device");

		Self {
			handle
		}
	}

	
	pub fn handle(&self) -> MochiAudioDeviceHandle {
		self.handle
	}


	pub fn start(&mut self) -> bool {
		unsafe { mochi_ahi_device_start(self.handle) }
	}

	pub fn stop(&mut self) {
		unsafe { mochi_ahi_device_stop(self.handle) }
	}


	pub fn is_playing(&self) -> bool {
		unsafe { mochi_ahi_device_is_playing(self.handle) }
	}
}

impl Drop for Device {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_ahi_device_stop(self.handle);
				mochi_ahi_device_dis(self.handle);
			}
		}
	}
}
