/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ops::{Deref, DerefMut};
use crate::error::RhiResult;
use crate::traits::{RhiBackend, RhiBuffer, RhiCommandBuffer};
use crate::types::{ClearColor, IndexType, Rect2D, ShaderStage, Viewport};


/// RAII Scope Guard for Dynamic Rendering
pub struct RenderingPass<'a, B: RhiBackend> {
	cmd: &'a mut B::CommandBuffer,
	target: &'a B::RenderTarget,
	active: bool,
}

impl<'a, B: RhiBackend> RenderingPass<'a, B> {
	pub fn new(
		cmd: &'a mut B::CommandBuffer,
		target: &'a B::RenderTarget,
		clear_color: impl Into<ClearColor>,
	) -> Self {
		cmd.begin_rendering(target, clear_color.into());
		Self {
			cmd,
			target,
			active: true,
		}
	}

	pub fn set_viewport(&mut self, vp: Viewport) {
		self.cmd.set_viewport(vp);
	}

	pub fn set_scissor(&mut self, scissor: Rect2D) {
		self.cmd.set_scissor(scissor);
	}

	pub fn bind_pipeline(&mut self, pipeline: &B::Pipeline) {
		self.cmd.bind_pipeline(pipeline);
	}

	pub fn bind_descriptor_heap(
		&mut self,
		res_heap: Option<&B::Buffer>,
		samp_heap: Option<&B::Buffer>,
		res_sz: u64,
		samp_sz: u64,
	) {
		self
			.cmd
			.bind_descriptor_heap(res_heap, samp_heap, res_sz, samp_sz);
	}

	pub fn push_constant_bytes(&mut self, stages: ShaderStage, offset: u32, bytes: &[u8]) {
		self.cmd.push_constant(stages, offset, bytes);
	}

	pub fn push_constant<T: Copy>(&mut self, stages: ShaderStage, offset: u32, value: &T) {
		let size = std::mem::size_of::<T>();
		let ptr = value as *const T as *const u8;
		let slice = unsafe { std::slice::from_raw_parts(ptr, size) };
		self.push_constant_bytes(stages, offset, slice);
	}

	pub fn bind_index_buffer(
		&mut self,
		buffer: &crate::Buffer<B>,
		offset: u64,
		index_type: IndexType,
	) {
		self.cmd.bind_index_buffer(buffer.raw(), offset, index_type);
	}

	pub fn draw(
		&mut self,
		vertex_count: u32,
		instance_count: u32,
		first_vertex: u32,
		first_instance: u32,
	) {
		self
			.cmd
			.draw(vertex_count, instance_count, first_vertex, first_instance);
	}

	pub fn draw_indexed(
		&mut self,
		index_count: u32,
		instance_count: u32,
		first_index: u32,
		vertex_offset: i32,
		first_instance: u32,
	) {
		self.cmd.draw_indexed(
			index_count,
			instance_count,
			first_index,
			vertex_offset,
			first_instance,
		);
	}

	pub fn finish(mut self) {
		if self.active {
			self.cmd.end_rendering(self.target);
			self.active = false;
		}
	}
}

impl<'a, B: RhiBackend> Drop for RenderingPass<'a, B> {
	fn drop(&mut self) {
		if self.active {
			self.cmd.end_rendering(self.target);
			self.active = false;
		}
	}
}


/// RAII Scope Guard for Mapped GPU Buffers
pub struct MappedBufferGuard<'a, B: RhiBuffer> {
	buffer: &'a mut B,
	slice: &'a mut [u8],
}

impl<'a, B: RhiBuffer> MappedBufferGuard<'a, B> {
	pub fn new(buffer: &'a mut B) -> RhiResult<Self> {
		let size = buffer.size() as usize;
		let ptr = buffer.map()?;
		let slice = unsafe { std::slice::from_raw_parts_mut(ptr, size) };
		Ok(Self { buffer, slice })
	}

	pub fn flush(&mut self, offset: u64, size: u64) -> RhiResult<()> {
		self.buffer.flush(offset, size)
	}
}

impl<'a, B: RhiBuffer> Deref for MappedBufferGuard<'a, B> {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		self.slice
	}
}

impl<'a, B: RhiBuffer> DerefMut for MappedBufferGuard<'a, B> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.slice
	}
}

impl<'a, B: RhiBuffer> Drop for MappedBufferGuard<'a, B> {
	fn drop(&mut self) {
		self.buffer.unmap();
	}
}
