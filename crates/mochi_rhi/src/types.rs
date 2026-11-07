/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_math::Vec4;
pub use crate::format::Format;
pub use crate::buffer::BufferUsage;
pub use crate::image::{ImageTiling, ImageUsage};
pub use crate::sampler::{SamplerAddressMode, SamplerFilter};
pub use crate::shader::ShaderStage;
pub use crate::pipeline::{PolygonMode, PrimitiveTopology, PipelineKind, VertexInputRate, DescriptorType};
pub use crate::manager::{AllocationCreate, AllocationLocation};


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Viewport {
	pub x: f32,
	pub y: f32,
	pub width: f32,
	pub height: f32,
	pub min_depth: f32,
	pub max_depth: f32,
}

impl Viewport {
	pub fn new(x: f32, y: f32, width: f32, height: f32, min_depth: f32, max_depth: f32) -> Self {
		Self { x, y, width, height, min_depth, max_depth }
	}

	pub fn from_size(width: f32, height: f32) -> Self {
		Self {
			x: 0.0,
			y: 0.0,
			width,
			height,
			min_depth: 0.0,
			max_depth: 1.0,
		}
	}
}


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Rect2D {
	pub x: i32,
	pub y: i32,
	pub width: u32,
	pub height: u32,
}

impl Rect2D {
	pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
		Self { x, y, width, height }
	}

	pub fn from_size(width: u32, height: u32) -> Self {
		Self { x: 0, y: 0, width, height }
	}
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ClearColor(pub [f32; 4]);

impl From<[f32; 4]> for ClearColor {
	fn from(arr: [f32; 4]) -> Self {
		Self(arr)
	}
}

impl From<Vec4<f32>> for ClearColor {
	fn from(v: Vec4<f32>) -> Self {
		Self([v.x, v.y, v.z, v.w])
	}
}
