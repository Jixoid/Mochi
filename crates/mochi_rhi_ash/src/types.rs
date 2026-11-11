/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use ash::vk;

use mochi_rhi::format::Format;
use mochi_rhi::pipeline::{PipelineKind, PolygonMode, PrimitiveTopology};
use mochi_rhi::sampler::{SamplerAddressMode, SamplerFilter};
use mochi_rhi::shader::ShaderStage;
use mochi_rhi::{BufferUsage, ImageTiling, ImageUsage};


#[inline]
pub fn to_vk_format(f: Format) -> vk::Format {
	vk::Format::from_raw(f as i32)
}

#[inline]
pub fn from_vk_format(f: vk::Format) -> Format {
	Format::from_raw(f.as_raw() as u32)
}

#[inline]
pub fn to_vk_buffer_usage(u: BufferUsage) -> vk::BufferUsageFlags {
	vk::BufferUsageFlags::from_raw(u.bits())
}

#[inline]
pub fn to_vk_image_usage(u: ImageUsage) -> vk::ImageUsageFlags {
	vk::ImageUsageFlags::from_raw(u.bits())
}

#[inline]
pub fn to_vk_image_tiling(t: ImageTiling) -> vk::ImageTiling {
	match t {
		ImageTiling::Optimal => vk::ImageTiling::OPTIMAL,
		ImageTiling::Linear => vk::ImageTiling::LINEAR,
	}
}

#[inline]
pub fn to_vk_address_mode(m: SamplerAddressMode) -> vk::SamplerAddressMode {
	match m {
		SamplerAddressMode::Repeat => vk::SamplerAddressMode::REPEAT,
		SamplerAddressMode::MirroredRepeat => vk::SamplerAddressMode::MIRRORED_REPEAT,
		SamplerAddressMode::ClampToEdge => vk::SamplerAddressMode::CLAMP_TO_EDGE,
		SamplerAddressMode::ClampToBorder => vk::SamplerAddressMode::CLAMP_TO_BORDER,
		SamplerAddressMode::MirrorClampToEdge => vk::SamplerAddressMode::MIRROR_CLAMP_TO_EDGE,
	}
}

#[inline]
pub fn to_vk_filter(f: SamplerFilter) -> vk::Filter {
	match f {
		SamplerFilter::Nearest => vk::Filter::NEAREST,
		SamplerFilter::Linear => vk::Filter::LINEAR,
		SamplerFilter::Cubic => vk::Filter::CUBIC_IMG,
	}
}

#[inline]
pub fn to_vk_shader_stage(s: ShaderStage) -> vk::ShaderStageFlags {
	vk::ShaderStageFlags::from_raw(s.bits())
}

#[inline]
pub fn to_vk_polygon_mode(m: PolygonMode) -> vk::PolygonMode {
	match m {
		PolygonMode::Fill => vk::PolygonMode::FILL,
		PolygonMode::Line => vk::PolygonMode::LINE,
		PolygonMode::Point => vk::PolygonMode::POINT,
	}
}

#[inline]
pub fn to_vk_primitive_topology(t: PrimitiveTopology) -> vk::PrimitiveTopology {
	match t {
		PrimitiveTopology::PointList => vk::PrimitiveTopology::POINT_LIST,
		PrimitiveTopology::LineList => vk::PrimitiveTopology::LINE_LIST,
		PrimitiveTopology::LineStrip => vk::PrimitiveTopology::LINE_STRIP,
		PrimitiveTopology::TriangleList => vk::PrimitiveTopology::TRIANGLE_LIST,
		PrimitiveTopology::TriangleStrip => vk::PrimitiveTopology::TRIANGLE_STRIP,
		PrimitiveTopology::TriangleFan => vk::PrimitiveTopology::TRIANGLE_FAN,
		PrimitiveTopology::PatchList => vk::PrimitiveTopology::PATCH_LIST,
	}
}

#[inline]
pub fn to_vk_pipeline_bind_point(k: PipelineKind) -> vk::PipelineBindPoint {
	match k {
		PipelineKind::Graphic => vk::PipelineBindPoint::GRAPHICS,
		PipelineKind::Compute => vk::PipelineBindPoint::COMPUTE,
		PipelineKind::RayTracing => vk::PipelineBindPoint::RAY_TRACING_KHR,
	}
}

#[inline]
pub fn to_vk_index_type(it: mochi_rhi::IndexType) -> vk::IndexType {
	match it {
		mochi_rhi::IndexType::Uint16 => vk::IndexType::UINT16,
		mochi_rhi::IndexType::Uint32 => vk::IndexType::UINT32,
	}
}

#[inline]
pub fn to_vk_sample_count(samples: mochi_rhi::MsaaSamples) -> vk::SampleCountFlags {
	match samples {
		mochi_rhi::MsaaSamples::X2 => vk::SampleCountFlags::TYPE_2,
		mochi_rhi::MsaaSamples::X4 => vk::SampleCountFlags::TYPE_4,
		mochi_rhi::MsaaSamples::X8 => vk::SampleCountFlags::TYPE_8,
		_ => vk::SampleCountFlags::TYPE_1,
	}
}
