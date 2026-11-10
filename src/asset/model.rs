/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::fs;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::material::{MaterialInstance, MaterialTemplate, PbrProperties};
use crate::rhi;
use crate::scene::{Mesh, Vertex};
use mochi_math::{Vec3, Vec4};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VertexAsset {
	pub pos: [f32; 4],
	pub normal: [f32; 4],
	pub tangent: [f32; 4],
	pub color: [f32; 4],
	pub uv0: [f32; 2],
	pub uv1: [f32; 2],
}

impl From<&Vertex> for VertexAsset {
	fn from(v: &Vertex) -> Self {
		Self {
			pos: v.pos,
			normal: v.normal,
			tangent: v.tangent,
			color: v.color,
			uv0: v.uv0,
			uv1: v.uv1,
		}
	}
}

impl From<&VertexAsset> for Vertex {
	fn from(v: &VertexAsset) -> Self {
		Self {
			pos: v.pos,
			normal: v.normal,
			tangent: v.tangent,
			color: v.color,
			uv0: v.uv0,
			uv1: v.uv1,
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmeshAsset {
	pub name: String,
	pub vertices: Vec<VertexAsset>,
	pub indices: Vec<u32>,
	pub material_index: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialAsset {
	pub name: String,
	pub base_color: [f32; 4],
	pub roughness: f32,
	pub metallic: f32,
	pub normal_scale: f32,
	pub occlusion: f32,
	pub emissive: [f32; 3],
	pub emissive_intensity: f32,
	pub albedo_texture: Option<String>,
	pub normal_texture: Option<String>,
	pub orm_texture: Option<String>,
}

impl Default for MaterialAsset {
	fn default() -> Self {
		Self {
			name: "DefaultMaterial".to_string(),
			base_color: [1.0, 1.0, 1.0, 1.0],
			roughness: 0.5,
			metallic: 0.0,
			normal_scale: 1.0,
			occlusion: 1.0,
			emissive: [0.0, 0.0, 0.0],
			emissive_intensity: 0.0,
			albedo_texture: None,
			normal_texture: None,
			orm_texture: None,
		}
	}
}

impl MaterialAsset {
	pub fn to_pbr_properties(&self) -> PbrProperties {
		PbrProperties {
			base_color: Vec4::new_with(self.base_color[0], self.base_color[1], self.base_color[2], self.base_color[3]),
			emissive: Vec3::new(self.emissive[0], self.emissive[1], self.emissive[2]),
			emissive_intensity: self.emissive_intensity,
			metallic: self.metallic,
			roughness: self.roughness,
			normal_scale: self.normal_scale,
			occlusion: self.occlusion,
			alpha_cutoff: 0.5,
			albedo_texture_id: None,
			normal_texture_id: None,
			orm_texture_id: None,
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelAsset {
	pub name: String,
	pub submeshes: Vec<SubmeshAsset>,
	pub materials: Vec<MaterialAsset>,
}

impl ModelAsset {
	pub fn load(path: &str) -> Result<Self, String> {
		if path.ends_with(".masset") {
			let bytes = fs::read(path).map_err(|e| format!("Failed to read binary asset {path}: {e}"))?;
			bincode::deserialize::<Self>(&bytes).map_err(|e| format!("Failed to deserialize binary asset {path}: {e}"))
		} else {
			let content = fs::read_to_string(path).map_err(|e| format!("Failed to read RON asset {path}: {e}"))?;
			ron::from_str::<Self>(&content).map_err(|e| format!("Failed to deserialize RON asset {path}: {e}"))
		}
	}

	pub fn save_ron(&self, path: &str) -> Result<(), String> {
		let s = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
			.map_err(|e| format!("Failed to serialize model to RON: {e}"))?;
		fs::write(path, s).map_err(|e| format!("Failed to write RON file {path}: {e}"))
	}

	pub fn save_binary(&self, path: &str) -> Result<(), String> {
		let bytes = bincode::serialize(self)
			.map_err(|e| format!("Failed to serialize model to binary: {e}"))?;
		fs::write(path, bytes).map_err(|e| format!("Failed to write binary file {path}: {e}"))
	}

	pub fn build_runtime(
		&self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		default_template: &Arc<MaterialTemplate>,
	) -> Vec<(Arc<Mesh>, Arc<MaterialInstance>)> {
		let material_instances: Vec<Arc<MaterialInstance>> = self
			.materials
			.iter()
			.map(|m| MaterialInstance::new(Arc::clone(default_template), m.to_pbr_properties()))
			.collect();

		let fallback_mat = MaterialInstance::new_default(Arc::clone(default_template));

		self.submeshes
			.iter()
			.map(|sm| {
				let vertices: Vec<Vertex> = sm.vertices.iter().map(Vertex::from).collect();
				let mesh = if sm.indices.is_empty() {
					Mesh::new_unindexed(device, alloc, &vertices)
				} else {
					Mesh::new_with_indices(device, alloc, &vertices, &sm.indices)
				};

				let mat = material_instances
					.get(sm.material_index)
					.cloned()
					.unwrap_or_else(|| Arc::clone(&fallback_mat));

				(mesh, mat)
			})
			.collect()
	}
}
