/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::material::{MaterialInstance, MaterialTemplate, PbrProperties, TextureManager};
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
#[serde(default)]
pub struct MaterialAsset {
	pub name: String,
	pub base_color: [f32; 4],
	pub roughness: f32,
	pub metallic: f32,
	pub normal_scale: f32,
	pub occlusion: f32,
	#[serde(default)]
	pub alpha_cutoff: f32,
	pub emissive: [f32; 3],
	pub emissive_intensity: f32,
	pub albedo_texture: Option<String>,
	pub normal_texture: Option<String>,
	pub orm_texture: Option<String>,
	pub emissive_texture: Option<String>,
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
			alpha_cutoff: 0.0,
			emissive: [0.0, 0.0, 0.0],
			emissive_intensity: 0.0,
			albedo_texture: None,
			normal_texture: None,
			orm_texture: None,
			emissive_texture: None,
		}
	}
}

impl MaterialAsset {
	pub fn to_pbr_properties(&self) -> PbrProperties {
		PbrProperties {
			base_color: Vec4::new_with(
				self.base_color[0],
				self.base_color[1],
				self.base_color[2],
				self.base_color[3],
			),
			emissive: Vec3::new(self.emissive[0], self.emissive[1], self.emissive[2]),
			emissive_intensity: self.emissive_intensity,
			metallic: self.metallic,
			roughness: self.roughness,
			normal_scale: self.normal_scale,
			occlusion: self.occlusion,
			alpha_cutoff: self.alpha_cutoff,
			albedo_texture_id: None,
			normal_texture_id: None,
			orm_texture_id: None,
			emissive_texture_id: None,
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelAsset {
	pub name: String,
	#[serde(default)]
	pub source: Option<String>,
	#[serde(default)]
	pub submeshes: Vec<SubmeshAsset>,
	#[serde(default)]
	pub materials: Vec<MaterialAsset>,
	#[serde(default)]
	pub exclude_meshes: Vec<String>,
}

impl ModelAsset {
	pub fn load(path: &str) -> Result<Self, String> {
		if path.ends_with(".glb") || path.ends_with(".gltf") {
			return Self::load_gltf(path);
		}

		if path.ends_with(".masset") {
			let bytes = fs::read(path).map_err(|e| format!("Failed to read binary asset {path}: {e}"))?;
			postcard::from_bytes::<Self>(&bytes)
				.map_err(|e| format!("Failed to deserialize binary asset {path}: {e}"))
		} else {
			let content =
				fs::read_to_string(path).map_err(|e| format!("Failed to read RON asset {path}: {e}"))?;
			let desc = ron::from_str::<Self>(&content)
				.map_err(|e| format!("Failed to deserialize RON asset {path}: {e}"))?;

			if let Some(src) = &desc.source {
				let resolved_source = if Path::new(src).is_absolute() {
					PathBuf::from(src)
				} else {
					let parent = Path::new(path).parent().unwrap_or(Path::new(""));
					let rel = parent.join(src);
					if rel.exists() {
						rel
					} else {
						PathBuf::from(src)
					}
				};

				let mut imported = Self::load_gltf(&resolved_source)?;
				if !desc.name.is_empty() {
					imported.name = desc.name;
				}
				if !desc.materials.is_empty() {
					imported.materials = desc.materials;
				}
				if !desc.exclude_meshes.is_empty() {
					imported.submeshes.retain(|s| {
						!desc
							.exclude_meshes
							.iter()
							.any(|exc| s.name.starts_with(exc))
					});
					imported.exclude_meshes = desc.exclude_meshes;
				}
				Ok(imported)
			} else {
				Ok(desc)
			}
		}
	}

	pub fn load_gltf(path: impl AsRef<Path>) -> Result<Self, String> {
		let path_ref = path.as_ref();
		let base = path_ref.parent().unwrap_or_else(|| Path::new("./"));
		let file = fs::File::open(path_ref)
			.map_err(|e| format!("Failed to open glTF {}: {e}", path_ref.display()))?;
		let reader = std::io::BufReader::new(file);
		let gltf = gltf::Gltf::from_reader(reader)
			.map_err(|e| format!("Failed to parse glTF {}: {e}", path_ref.display()))?;
		let (document, blob) = (gltf.document, gltf.blob);
		let buffers = gltf::import_buffers(&document, Some(base), blob)
			.map_err(|e| format!("Failed to import buffers for {}: {e}", path_ref.display()))?;

		let mut materials = Vec::new();

		for mat in document.materials() {
			let pbr = mat.pbr_metallic_roughness();
			let mut base_color = pbr.base_color_factor();
			let mut roughness = pbr.roughness_factor();
			if let Some(trans) = mat.transmission() {
				let factor = trans.transmission_factor();
				if factor > 0.0 {
					base_color[3] = (base_color[3] * (1.0 - factor * 0.82)).clamp(0.12, 1.0);
					roughness = roughness.min(0.08);
				}
			}
			if matches!(mat.alpha_mode(), gltf::material::AlphaMode::Blend) && base_color[3] >= 0.999 {
				base_color[3] = 0.5;
			}
			let metallic = pbr.metallic_factor();
			let emissive = if mat.emissive_texture().is_some() {
				[0.0, 0.0, 0.0]
			} else {
				mat.emissive_factor()
			};
			let normal_scale = mat.normal_texture().map_or(1.0, |t| t.scale());
			let occlusion = mat.occlusion_texture().map_or(1.0, |t| t.strength());
			let alpha_cutoff = match mat.alpha_mode() {
				gltf::material::AlphaMode::Mask => mat.alpha_cutoff().unwrap_or(0.5),
				_ => 0.0,
			};

			materials.push(MaterialAsset {
				name: mat.name().unwrap_or("Material").to_string(),
				base_color,
				roughness,
				metallic,
				normal_scale,
				occlusion,
				alpha_cutoff,
				emissive,
				emissive_intensity: 1.0,
				albedo_texture: None,
				normal_texture: None,
				orm_texture: None,
				emissive_texture: None,
			});
		}

		if materials.is_empty() {
			materials.push(MaterialAsset::default());
		}

		let mut submeshes = Vec::new();

		for mesh in document.meshes() {
			if mesh.name() == Some("Fabric") {
				continue;
			}
			for (prim_idx, primitive) in mesh.primitives().enumerate() {
				let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

				let positions: Vec<[f32; 3]> = match reader.read_positions() {
					Some(iter) => iter.collect(),
					None => continue,
				};

				let normals: Vec<[f32; 3]> = reader
					.read_normals()
					.map(|iter| iter.collect())
					.unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

				let tangents: Vec<[f32; 4]> = reader
					.read_tangents()
					.map(|iter| iter.collect())
					.unwrap_or_else(|| vec![[1.0, 0.0, 0.0, 1.0]; positions.len()]);

				let colors: Vec<[f32; 4]> = reader
					.read_colors(0)
					.map(|iter| iter.into_rgba_f32().collect())
					.unwrap_or_else(|| vec![[1.0, 1.0, 1.0, 1.0]; positions.len()]);

				let uvs: Vec<[f32; 2]> = reader
					.read_tex_coords(0)
					.map(|iter| iter.into_f32().collect())
					.unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

				let indices: Vec<u32> = reader
					.read_indices()
					.map(|iter| iter.into_u32().collect())
					.unwrap_or_else(|| (0..positions.len() as u32).collect());

				let mut vertices = Vec::with_capacity(positions.len());
				for (i, &p) in positions.iter().enumerate() {
					let n = normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]);
					let t = tangents.get(i).copied().unwrap_or([1.0, 0.0, 0.0, 1.0]);
					let c = colors.get(i).copied().unwrap_or([1.0, 1.0, 1.0, 1.0]);
					let uv = uvs.get(i).copied().unwrap_or([0.0, 0.0]);

					vertices.push(VertexAsset {
						pos: [p[0], p[1], p[2], 1.0],
						normal: [n[0], n[1], n[2], 0.0],
						tangent: t,
						color: c,
						uv0: uv,
						uv1: [0.0, 0.0],
					});
				}

				let material_index = primitive.material().index().unwrap_or(0);
				let prim_name = format!("{}_{prim_idx}", mesh.name().unwrap_or("Mesh"));

				submeshes.push(SubmeshAsset {
					name: prim_name,
					vertices,
					indices,
					material_index,
				});
			}
		}

		let model_name = path_ref
			.file_stem()
			.and_then(|s| s.to_str())
			.unwrap_or("GltfModel")
			.to_string();

		Ok(Self {
			name: model_name,
			source: Some(path_ref.to_string_lossy().to_string()),
			submeshes,
			materials,
			exclude_meshes: Vec::new(),
		})
	}

	pub fn save_ron(&self, path: &str) -> Result<(), String> {
		let s = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
			.map_err(|e| format!("Failed to serialize model to RON: {e}"))?;
		fs::write(path, s).map_err(|e| format!("Failed to write RON file {path}: {e}"))
	}

	pub fn save_binary(&self, path: &str) -> Result<(), String> {
		let bytes =
			postcard::to_stdvec(self).map_err(|e| format!("Failed to serialize model to binary: {e}"))?;
		fs::write(path, bytes).map_err(|e| format!("Failed to write binary file {path}: {e}"))
	}

	pub fn build_runtime(
		&self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		cmdmng: &rhi::CommandManager,
		texture_manager: Option<&mut TextureManager>,
		default_template: &Arc<MaterialTemplate>,
		transparent_template: &Arc<MaterialTemplate>,
	) -> Vec<(Arc<Mesh>, Arc<MaterialInstance>)> {
		let mut material_instances: Vec<Arc<MaterialInstance>> = Vec::new();

		if let (Some(source_path), Some(ref mut tm)) = (&self.source, texture_manager)
			&& (source_path.ends_with(".glb") || source_path.ends_with(".gltf"))
			&& Path::new(source_path).exists()
			&& let Ok((document, _buffers, images)) = gltf::import(source_path)
		{
			let decode_image_rgba8 = |img_data: &gltf::image::Data| -> Vec<u8> {
				match img_data.format {
					gltf::image::Format::R8G8B8A8 => img_data.pixels.clone(),
					gltf::image::Format::R8G8B8 => {
						let mut out = Vec::with_capacity((img_data.width * img_data.height * 4) as usize);
						for c in img_data.pixels.as_chunks::<3>().0 {
							out.extend_from_slice(&[c[0], c[1], c[2], 255]);
						}
						out
					}
					gltf::image::Format::R8 => {
						let mut out = Vec::with_capacity((img_data.width * img_data.height * 4) as usize);
						for &p in &img_data.pixels {
							out.extend_from_slice(&[p, p, p, 255]);
						}
						out
					}
					_ => img_data.pixels.clone(),
				}
			};

			let mut uploaded_textures: HashMap<(usize, mochi_rhi::Format), u32> = HashMap::new();

			let get_or_upload = |uploaded: &mut HashMap<(usize, mochi_rhi::Format), u32>,
			                     tm: &mut TextureManager,
			                     img_idx: usize,
			                     fmt: mochi_rhi::Format|
			 -> u32 {
				if let Some(&id) = uploaded.get(&(img_idx, fmt)) {
					return id;
				}
				if let Some(img_data) = images.get(img_idx) {
					let rgba = decode_image_rgba8(img_data);
					let id = tm.load_raw_rgba8(
						device,
						alloc,
						cmdmng,
						img_data.width,
						img_data.height,
						fmt,
						&rgba,
					);
					uploaded.insert((img_idx, fmt), id);
					id
				} else {
					0
				}
			};

			for (mat_idx, doc_mat) in document.materials().enumerate() {
				let pbr = doc_mat.pbr_metallic_roughness();

				let albedo_id = pbr.base_color_texture().map(|t| {
					get_or_upload(
						&mut uploaded_textures,
						tm,
						t.texture().source().index(),
						mochi_rhi::Format::RGBA8_SRGB,
					)
				});

				let normal_id = doc_mat.normal_texture().map(|t| {
					get_or_upload(
						&mut uploaded_textures,
						tm,
						t.texture().source().index(),
						mochi_rhi::Format::RGBA8_UNORM,
					)
				});

				let mr_tex = pbr.metallic_roughness_texture();
				let occ_tex = doc_mat.occlusion_texture();
				let orm_id = match (occ_tex, mr_tex) {
					(Some(occ), Some(mr))
						if occ.texture().source().index() == mr.texture().source().index() =>
					{
						Some(get_or_upload(
							&mut uploaded_textures,
							tm,
							occ.texture().source().index(),
							mochi_rhi::Format::RGBA8_UNORM,
						))
					}
					(Some(occ), Some(mr)) => {
						let occ_idx = occ.texture().source().index();
						let mr_idx = mr.texture().source().index();
						let key = (usize::MAX - 1 - mat_idx, mochi_rhi::Format::RGBA8_UNORM);
						if let Some(&id) = uploaded_textures.get(&key) {
							Some(id)
						} else if let (Some(occ_img), Some(mr_img)) = (images.get(occ_idx), images.get(mr_idx))
						{
							let occ_rgba = decode_image_rgba8(occ_img);
							let mr_rgba = decode_image_rgba8(mr_img);
							let w = mr_img.width;
							let h = mr_img.height;
							let pixel_count = (w * h) as usize;
							let mut combined = Vec::with_capacity(pixel_count * 4);
							for i in 0..pixel_count {
								let r = occ_rgba.get(i * 4).copied().unwrap_or(255);
								let g = mr_rgba.get(i * 4 + 1).copied().unwrap_or(255);
								let b = mr_rgba.get(i * 4 + 2).copied().unwrap_or(255);
								combined.extend_from_slice(&[r, g, b, 255]);
							}
							let id = tm.load_raw_rgba8(
								device,
								alloc,
								cmdmng,
								w,
								h,
								mochi_rhi::Format::RGBA8_UNORM,
								&combined,
							);
							uploaded_textures.insert(key, id);
							Some(id)
						} else {
							None
						}
					}
					(None, Some(mr)) => Some(get_or_upload(
						&mut uploaded_textures,
						tm,
						mr.texture().source().index(),
						mochi_rhi::Format::RGBA8_UNORM,
					)),
					(Some(occ), None) => Some(get_or_upload(
						&mut uploaded_textures,
						tm,
						occ.texture().source().index(),
						mochi_rhi::Format::RGBA8_UNORM,
					)),
					(None, None) => None,
				};

				let emissive_id = doc_mat.emissive_texture().map(|t| {
					get_or_upload(
						&mut uploaded_textures,
						tm,
						t.texture().source().index(),
						mochi_rhi::Format::RGBA8_SRGB,
					)
				});

				let mut props = self
					.materials
					.get(mat_idx)
					.map(|m| m.to_pbr_properties())
					.unwrap_or_default();
				props.albedo_texture_id = albedo_id;
				props.normal_texture_id = normal_id;
				props.orm_texture_id = orm_id;
				props.emissive_texture_id = emissive_id;

				crate::mochi_disp!(
					"Model '{}': Material '{}' bound textures: albedo={:?}, normal={:?}, orm={:?}, emissive={:?}",
					self.name,
					doc_mat.name().unwrap_or("Material"),
					albedo_id,
					normal_id,
					orm_id,
					emissive_id
				);

				let tmpl = if props.base_color.w < 0.999 {
					props.alpha_cutoff = 0.0;
					Arc::clone(transparent_template)
				} else {
					Arc::clone(default_template)
				};

				material_instances.push(MaterialInstance::new(tmpl, props));
			}
		}

		if material_instances.is_empty() {
			material_instances = self
				.materials
				.iter()
				.map(|m| {
					let mut props = m.to_pbr_properties();
					let tmpl = if props.base_color.w < 0.999 {
						props.alpha_cutoff = 0.0;
						Arc::clone(transparent_template)
					} else {
						Arc::clone(default_template)
					};
					MaterialInstance::new(tmpl, props)
				})
				.collect();
		}

		let fallback_mat = MaterialInstance::new_default(Arc::clone(default_template));

		self
			.submeshes
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
