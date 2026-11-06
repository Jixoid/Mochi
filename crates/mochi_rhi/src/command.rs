/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::ffi::*;
use crate::render_target::RenderTarget;


pub struct Command {
	handle: MochiCommandHandle,
}

impl Command {
	pub fn from_handle(handle: MochiCommandHandle) -> Self {
		Self { handle }
	}

	pub fn handle(&self) -> MochiCommandHandle {
		self.handle
	}

	
	pub fn begin(&self) {
		unsafe { mochi_rhi_command_begin(self.handle) }
	}

	pub fn end(&self) {
		unsafe { mochi_rhi_command_end(self.handle) }
	}


	pub fn begin_rendering(&self, target: &RenderTarget, clear_color: [f32; 4]) {
		unsafe {
			mochi_rhi_command_begin_rendering(
				self.handle,
				target.handle(),
				clear_color[0],
				clear_color[1],
				clear_color[2],
				clear_color[3],
			)
		}
	}

	pub fn end_rendering(&self, target: &RenderTarget) {
		unsafe {
			mochi_rhi_command_end_rendering(self.handle, target.handle())
		}
	}

	
	pub fn set_viewport(&self, x: f32, y: f32, width: f32, height: f32, min_depth: f32, max_depth: f32) {
		unsafe {
			mochi_rhi_command_set_viewport(self.handle, x, y, width, height, min_depth, max_depth)
		}
	}

	pub fn set_scissor(&self, x: i32, y: i32, width: u32, height: u32) {
		unsafe {
			mochi_rhi_command_set_scissor(self.handle, x, y, width, height)
		}
	}


	pub fn bind_pipeline(&self, pipeline: &crate::Pipeline) {
		unsafe { mochi_rhi_command_bind_pipeline(self.handle, pipeline.handle()) }
	}

	pub fn bind_descriptor_heap(&self, resource_heap: Option<&crate::Buffer>, sampler_heap: Option<&crate::Buffer>, resource_size: u64, sampler_size: u64) {
		let res_h = resource_heap.map(|b| b.handle()).unwrap_or(std::ptr::null_mut());
		let samp_h = sampler_heap.map(|b| b.handle()).unwrap_or(std::ptr::null_mut());
		
		unsafe {
			mochi_rhi_command_bind_descriptor_heap(self.handle, res_h, samp_h, resource_size, sampler_size);
		}
	}

	pub fn push_constant_bytes(&self, stage: crate::ShaderStage, offset: u32, bytes: &[u8]) {
		unsafe {
			mochi_rhi_command_push_constant(
				self.handle,
				stage.bits(),
				offset,
				bytes.len() as u32,
				bytes.as_ptr() as *const std::ffi::c_void,
			);
		}
	}

	pub fn push_constant<T: Copy>(&self, stage: crate::ShaderStage, offset: u32, value: &T) {
		let size = std::mem::size_of::<T>();
		let ptr = value as *const T as *const u8;
		let slice = unsafe { std::slice::from_raw_parts(ptr, size) };
		self.push_constant_bytes(stage, offset, slice);
	}

	
	pub fn draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
		unsafe {
			mochi_rhi_command_draw(self.handle, vertex_count, instance_count, first_vertex, first_instance);
		}
	}
}

impl Drop for Command {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_command_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}
