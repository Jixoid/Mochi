/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use mochi_math::Vec4;

use crate::ffi::*;
use crate::manager::command_manager::CommandManagerInner;
use crate::render_target::RenderTarget;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CommandState {
	Initial,
	Recording,
	Rendering,
	Executable,
}

pub struct Command {
	pool: Arc<CommandManagerInner>,
	handle: MochiCommandHandle,
	state: CommandState,
}


unsafe impl Send for Command {}

impl Command {
	pub fn from_handle(pool: Arc<CommandManagerInner>, handle: MochiCommandHandle) -> Self {
		Self {
			pool,
			handle,
			state: CommandState::Initial,
		}
	}

	pub fn state(&self) -> CommandState {
		self.state
	}

	pub fn pool(&self) -> &Arc<CommandManagerInner> {
		&self.pool
	}

	pub fn handle(&self) -> MochiCommandHandle {
		self.handle
	}

	pub fn begin(&mut self) {
		assert!(
			self.state == CommandState::Initial || self.state == CommandState::Executable,
			"Cannot begin recording: Command is already recording or inside a render pass (current state: {:?})",
			self.state
		);
		unsafe { mochi_rhi_command_begin(self.handle) }
		self.state = CommandState::Recording;
	}

	pub fn end(&mut self) {
		assert!(
			self.state == CommandState::Recording,
			"Cannot end command: Command is not recording or still inside an unclosed render pass (current state: {:?})",
			self.state
		);
		unsafe { mochi_rhi_command_end(self.handle) }
		self.state = CommandState::Executable;
	}

	pub fn begin_rendering(&mut self, target: &RenderTarget<'_>, clear_color: impl Into<Vec4<f32>>) {
		assert!(
			self.state == CommandState::Recording,
			"Cannot begin rendering: Command is not recording or already rendering (current state: {:?})",
			self.state
		);
		let col = clear_color.into();
		unsafe {
			mochi_rhi_command_begin_rendering(
				self.handle,
				target.handle(),
				col.x,
				col.y,
				col.z,
				col.w,
			)
		}
		self.state = CommandState::Rendering;
	}

	pub fn end_rendering(&mut self, target: &RenderTarget<'_>) {
		assert!(
			self.state == CommandState::Rendering,
			"Cannot end rendering: Command is not in a rendering pass (current state: {:?})",
			self.state
		);
		unsafe {
			mochi_rhi_command_end_rendering(self.handle, target.handle())
		}
		self.state = CommandState::Recording;
	}

	
	pub fn render_pass<F, R>(&mut self, target: &RenderTarget<'_>, clear_color: impl Into<Vec4<f32>>, f: F) -> R
	where
		F: FnOnce(&mut Command) -> R,
	{
		self.begin_rendering(target, clear_color);
		let res = f(self);
		self.end_rendering(target);
		res
	}

	pub fn set_viewport(&mut self, x: f32, y: f32, width: f32, height: f32, min_depth: f32, max_depth: f32) {
		assert!(
			self.state == CommandState::Recording || self.state == CommandState::Rendering,
			"Cannot set_viewport: Command must be recording"
		);
		unsafe {
			mochi_rhi_command_set_viewport(self.handle, x, y, width, height, min_depth, max_depth)
		}
	}

	pub fn set_scissor(&mut self, x: i32, y: i32, width: u32, height: u32) {
		assert!(
			self.state == CommandState::Recording || self.state == CommandState::Rendering,
			"Cannot set_scissor: Command must be recording"
		);
		unsafe {
			mochi_rhi_command_set_scissor(self.handle, x, y, width, height)
		}
	}

	pub fn bind_pipeline(&mut self, pipeline: &crate::Pipeline) {
		assert!(
			self.state == CommandState::Recording || self.state == CommandState::Rendering,
			"Cannot bind_pipeline: Command must be recording"
		);
		unsafe { mochi_rhi_command_bind_pipeline(self.handle, pipeline.handle()) }
	}

	pub fn bind_descriptor_heap(&mut self, resource_heap: Option<&crate::Buffer>, sampler_heap: Option<&crate::Buffer>, resource_size: u64, sampler_size: u64) {
		assert!(
			self.state == CommandState::Recording || self.state == CommandState::Rendering,
			"Cannot bind_descriptor_heap: Command must be recording"
		);
		let res_h = resource_heap.map(|b| b.handle()).unwrap_or(std::ptr::null_mut());
		let samp_h = sampler_heap.map(|b| b.handle()).unwrap_or(std::ptr::null_mut());
		
		unsafe {
			mochi_rhi_command_bind_descriptor_heap(self.handle, res_h, samp_h, resource_size, sampler_size);
		}
	}

	pub fn push_constant_bytes(&mut self, stage: crate::ShaderStage, offset: u32, bytes: &[u8]) {
		assert!(
			self.state == CommandState::Recording || self.state == CommandState::Rendering,
			"Cannot push_constant: Command must be recording"
		);
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

	pub fn push_constant<T: Copy>(&mut self, stage: crate::ShaderStage, offset: u32, value: &T) {
		let size = std::mem::size_of::<T>();
		let ptr = value as *const T as *const u8;
		let slice = unsafe { std::slice::from_raw_parts(ptr, size) };
		self.push_constant_bytes(stage, offset, slice);
	}

	pub fn draw(&mut self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
		assert!(
			self.state == CommandState::Rendering,
			"Cannot draw: Command must be inside an active dynamic rendering pass (call render_pass or begin_rendering)"
		);
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
