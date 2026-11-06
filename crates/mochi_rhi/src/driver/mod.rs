/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ffi::CStr;
use crate::ffi::*;


pub struct Driver;

impl Driver {
	pub fn name() -> String {
		unsafe {
			let ptr = mochi_rhi_get_driver_name();
			if ptr.is_null() {
				"unknown".to_string()
			} else {
				CStr::from_ptr(ptr).to_string_lossy().into_owned()
			}
		}
	}

	pub fn version() -> [u32; 4] {
		let mut vers = [0u32; 4];
		unsafe {
			mochi_rhi_get_driver_version(vers.as_mut_ptr());
		}
		vers
	}
}
