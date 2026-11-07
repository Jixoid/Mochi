/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_math::Vec4;

use crate::pipeline::Pipeline;
use crate::render_target::RenderTarget;
use crate::shader::ShaderStage;
use crate::traits::{RhiBackend, RhiCommandBuffer};
use crate::types::{ClearColor, Rect2D, Viewport};
use crate::Buffer;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CommandState {
	Initial,
	Recording,
	Rendering,
	Executable,
}


pub struct Command<B: RhiBackend> {
	inner: B::CommandBuffer,
	state: CommandState,
}

unsafe impl<B: RhiBackend> Send for Command<B> {}

impl<B: RhiBackend> Command<B> {
	pub fn from_raw(inner: B::CommandBuffer) -> Self {
		Self {
			inner,
			state: CommandState::Initial,
		}
	}

	pub fn raw(&self) -> &B::CommandBuffer {
		&self.inner
	}

	pub fn raw_mut(&mut self) -> &mut B::CommandBuffer {
		&mut self.inner
	}

	pub fn state(&self) -> CommandState {
		self.state
	}

	pub fn begin(&mut self) {
		assert!(
			self.state == CommandState::Initial || self.state == CommandState::Executable,
			"Cannot begin recording: Command is already recording or inside a render pass (current state: {:?})",
			self.state
		);
		self.inner.begin().expect("Command buffer begin failed");
		self.state = CommandState::Recording;
	}

	pub fn end(&mut self) {
		assert!(
			self.state == CommandState::Recording,
			"Cannot end command: Command is not recording or still inside an unclosed render pass (current state: {:?})",
			self.state
		);
		self.inner.end().expect("Command buffer end failed");
		self.state = CommandState::Executable;
	}

	pub fn begin_rendering(&mut self, target: &RenderTarget<'_, B>, clear_color: impl Into<Vec4<f32>>) {
		assert!(
			self.state == CommandState::Recording,
			"Cannot begin rendering: Command is not recording or already rendering (current state: {:?})",
			self.state
		);
		let col = clear_color.into();
		self.inner.begin_rendering(target.raw(), ClearColor([col.x, col.y, col.z, col.w]));
		self.state = CommandState::Rendering;
	}

	pub fn end_rendering(&mut self, target: &RenderTarget<'_, B>) {
		assert!(
			self.state == CommandState::Rendering,
			"Cannot end rendering: Command is not in a rendering pass (current state: {:?})",
			self.state
		);
		self.inner.end_rendering(target.raw());
		self.state = CommandState::Recording;
	}

	pub fn render_pass<F, R>(&mut self, target: &RenderTarget<'_, B>, clear_color: impl Into<Vec4<f32>>, f: F) -> R
	where
		F: FnOnce(&mut Command<B>) -> R,
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
		self.inner.set_viewport(Viewport {
			x,
			y,
			width,
			height,
			min_depth,
			max_depth,
		});
	}

	pub fn set_scissor(&mut self, x: i32, y: i32, width: u32, height: u32) {
		assert!(
			self.state == CommandState::Recording || self.state == CommandState::Rendering,
			"Cannot set_scissor: Command must be recording"
		);
		self.inner.set_scissor(Rect2D {
			x,
			y,
			width,
			height,
		});
	}

	pub fn bind_pipeline(&mut self, pipeline: &Pipeline<B>) {
		assert!(
			self.state == CommandState::Recording || self.state == CommandState::Rendering,
			"Cannot bind_pipeline: Command must be recording"
		);
		self.inner.bind_pipeline(pipeline.raw());
	}

	pub fn bind_descriptor_heap(
		&mut self,
		resource_heap: Option<&Buffer<B>>,
		sampler_heap: Option<&Buffer<B>>,
		resource_size: u64,
		sampler_size: u64,
	) {
		assert!(
			self.state == CommandState::Recording || self.state == CommandState::Rendering,
			"Cannot bind_descriptor_heap: Command must be recording"
		);
		self.inner.bind_descriptor_heap(
			resource_heap.map(|b| b.raw()),
			sampler_heap.map(|b| b.raw()),
			resource_size,
			sampler_size,
		);
	}

	pub fn push_constant_bytes(&mut self, stage: ShaderStage, offset: u32, bytes: &[u8]) {
		assert!(
			self.state == CommandState::Recording || self.state == CommandState::Rendering,
			"Cannot push_constant: Command must be recording"
		);
		self.inner.push_constant(stage, offset, bytes);
	}

	pub fn push_constant<T: Copy>(&mut self, stage: ShaderStage, offset: u32, value: &T) {
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
		self.inner.draw(vertex_count, instance_count, first_vertex, first_instance);
	}
}
