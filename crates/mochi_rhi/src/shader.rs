/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ffi::CString;
use bitflags::bitflags;

use crate::Device;
use crate::ffi::*;


bitflags! {
	#[derive(Copy, Clone, Debug, PartialEq, Eq)]
	pub struct ShaderStage: u32 {
		const Vertex       = 0x00000001;
		const Pixel        = 0x00000010;
		const Compute      = 0x00000020;
		const Task         = 0x00000040;
		const Mesh         = 0x00000080;
		const RayGen       = 0x00000100;
		const AnyHit       = 0x00000200;
		const ClosestHit   = 0x00000400;
		const Miss         = 0x00000800;
		const Intersection = 0x00001000;
		const Callable     = 0x00002000;
	}
}


pub struct Shader {
	handle: MochiShaderHandle,
	stage: ShaderStage,
	entry: String,
}

impl Shader {
	pub fn new(device: &Device, stage: ShaderStage, code: &[u8], entry: &str) -> Self {
		let c_entry = CString::new(entry).expect("CString failed for shader entry");
		let handle = unsafe {
			mochi_rhi_shader_new(
				device.handle(),
				stage.bits(),
				code.as_ptr(),
				code.len(),
				c_entry.as_ptr(),
			)
		};
		assert!(!handle.is_null(), "RHI driver failed to create Shader");

		Self {
			handle,
			stage,
			entry: entry.to_string(),
		}
	}

	pub fn handle(&self) -> MochiShaderHandle {
		self.handle
	}

	pub fn stage(&self) -> ShaderStage {
		self.stage
	}

	pub fn entry(&self) -> &str {
		&self.entry
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_shader_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}
