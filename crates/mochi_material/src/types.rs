/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_math::{Vec3, Vec4};


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PassKind {
	DepthPrepass,
	ShadowCaster,
	ForwardLit,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BlendMode {
	Opaque,
	Masked { alpha_cutoff: f32 },
	Transparent,
	Additive,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CullMode {
	Back,
	Front,
	None,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PbrMaterialGpuData {
	pub base_color: [f32; 4],
	pub emissive: [f32; 4],
	pub roughness: f32,
	pub metallic: f32,
	pub normal_scale: f32,
	pub occlusion: f32,
	pub alpha_cutoff: f32,
	pub albedo_texture_id: u32,
	pub normal_texture_id: u32,
	pub orm_texture_id: u32,
	pub emissive_texture_id: u32,
	pub _pad: [u32; 3],
}

impl Default for PbrMaterialGpuData {
	fn default() -> Self {
		Self {
			base_color: [1.0, 1.0, 1.0, 1.0],
			emissive: [0.0, 0.0, 0.0, 0.0],
			roughness: 0.5,
			metallic: 0.0,
			normal_scale: 1.0,
			occlusion: 1.0,
			alpha_cutoff: 0.0,
			albedo_texture_id: u32::MAX,
			normal_texture_id: u32::MAX,
			orm_texture_id: u32::MAX,
			emissive_texture_id: u32::MAX,
			_pad: [0; 3],
		}
	}
}

#[derive(Clone, Debug)]
pub struct PbrProperties {
	pub base_color: Vec4<f32>,
	pub emissive: Vec3<f32>,
	pub emissive_intensity: f32,
	pub metallic: f32,
	pub roughness: f32,
	pub normal_scale: f32,
	pub occlusion: f32,
	pub alpha_cutoff: f32,
	pub albedo_texture_id: Option<u32>,
	pub normal_texture_id: Option<u32>,
	pub orm_texture_id: Option<u32>,
	pub emissive_texture_id: Option<u32>,
}

impl Default for PbrProperties {
	fn default() -> Self {
		Self {
			base_color: Vec4::new_with(1.0, 1.0, 1.0, 1.0),
			emissive: Vec3::ZERO,
			emissive_intensity: 0.0,
			metallic: 0.0,
			roughness: 0.5,
			normal_scale: 1.0,
			occlusion: 1.0,
			alpha_cutoff: 0.0,
			albedo_texture_id: None,
			normal_texture_id: None,
			orm_texture_id: None,
			emissive_texture_id: None,
		}
	}
}

impl PbrProperties {
	pub fn to_gpu_data(&self) -> PbrMaterialGpuData {
		PbrMaterialGpuData {
			base_color: [
				self.base_color.x,
				self.base_color.y,
				self.base_color.z,
				self.base_color.w,
			],
			emissive: [
				self.emissive.x,
				self.emissive.y,
				self.emissive.z,
				self.emissive_intensity,
			],
			roughness: self.roughness,
			metallic: self.metallic,
			normal_scale: self.normal_scale,
			occlusion: self.occlusion,
			alpha_cutoff: self.alpha_cutoff,
			albedo_texture_id: self.albedo_texture_id.unwrap_or(u32::MAX),
			normal_texture_id: self.normal_texture_id.unwrap_or(u32::MAX),
			orm_texture_id: self.orm_texture_id.unwrap_or(u32::MAX),
			emissive_texture_id: self.emissive_texture_id.unwrap_or(u32::MAX),
			_pad: [0; 3],
		}
	}
}
