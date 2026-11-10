/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::fs;
use serde::{Deserialize, Serialize};
use crate::asset::model::ModelAsset;
use crate::rendering::RenderingSystem;
use crate::rhi;
use crate::scene::{CameraComponent, DirectionalLightComponent, MeshRendererComponent, PointLightComponent, Scene, TransformComponent};
use mochi_except::*;
use mochi_math::{Quat, Transform, Vec3, Vec4};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransformDesc {
	#[serde(default = "default_translation")]
	pub translation: [f32; 3],
	#[serde(default = "default_rotation")]
	pub rotation: [f32; 4],
	#[serde(default = "default_scale")]
	pub scale: [f32; 3],
}

fn default_translation() -> [f32; 3] { [0.0, 0.0, 0.0] }
fn default_rotation() -> [f32; 4] { [0.0, 0.0, 0.0, 1.0] }
fn default_scale() -> [f32; 3] { [1.0, 1.0, 1.0] }

impl Default for TransformDesc {
	fn default() -> Self {
		Self {
			translation: default_translation(),
			rotation: default_rotation(),
			scale: default_scale(),
		}
	}
}

impl TransformDesc {
	pub fn to_transform(&self) -> Transform<f32> {
		Transform {
			translation: Vec3::new(self.translation[0], self.translation[1], self.translation[2]),
			rotation: Quat::new(self.rotation[0], self.rotation[1], self.rotation[2], self.rotation[3]),
			scale: Vec3::new(self.scale[0], self.scale[1], self.scale[2]),
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CameraDesc {
	#[serde(default = "default_fov")]
	pub fov_deg: f32,
	#[serde(default = "default_near")]
	pub near: f32,
	#[serde(default = "default_far")]
	pub far: f32,
	#[serde(default = "default_active")]
	pub is_active: bool,
	#[serde(default = "default_clear_color")]
	pub clear_color: [f32; 4],
}

fn default_fov() -> f32 { 60.0 }
fn default_near() -> f32 { 0.1 }
fn default_far() -> f32 { 1000.0 }
fn default_active() -> bool { true }
fn default_clear_color() -> [f32; 4] { [0.1, 0.15, 0.25, 1.0] }

impl Default for CameraDesc {
	fn default() -> Self {
		Self {
			fov_deg: default_fov(),
			near: default_near(),
			far: default_far(),
			is_active: default_active(),
			clear_color: default_clear_color(),
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectionalLightDesc {
	pub color: [f32; 3],
	pub intensity: f32,
	pub direction: [f32; 3],
	#[serde(default = "default_cast_shadows")]
	pub cast_shadows: bool,
}

fn default_cast_shadows() -> bool { true }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointLightDesc {
	pub color: [f32; 3],
	pub intensity: f32,
	pub radius: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeshRendererDesc {
	pub asset: String,
	#[serde(default = "default_visible")]
	pub is_visible: bool,
	#[serde(default = "default_cast_shadows")]
	pub cast_shadows: bool,
}

fn default_visible() -> bool { true }

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EntityDesc {
	pub name: Option<String>,
	pub transform: Option<TransformDesc>,
	pub camera: Option<CameraDesc>,
	pub directional_light: Option<DirectionalLightDesc>,
	pub point_light: Option<PointLightDesc>,
	pub mesh_renderer: Option<MeshRendererDesc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneAsset {
	pub name: String,
	pub entities: Vec<EntityDesc>,
}

impl SceneAsset {
	pub fn load(path: &str) -> Result<Self, String> {
		if path.ends_with(".mscene") {
			let bytes = fs::read(path).map_err(|e| format!("Failed to read binary scene {path}: {e}"))?;
			bincode::deserialize::<Self>(&bytes).map_err(|e| format!("Failed to deserialize binary scene {path}: {e}"))
		} else {
			let content = fs::read_to_string(path).map_err(|e| format!("Failed to read RON scene {path}: {e}"))?;
			ron::from_str::<Self>(&content).map_err(|e| format!("Failed to deserialize RON scene {path}: {e}"))
		}
	}

	pub fn save_ron(&self, path: &str) -> Result<(), String> {
		let s = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
			.map_err(|e| format!("Failed to serialize scene to RON: {e}"))?;
		fs::write(path, s).map_err(|e| format!("Failed to write RON file {path}: {e}"))
	}

	pub fn save_binary(&self, path: &str) -> Result<(), String> {
		let bytes = bincode::serialize(self)
			.map_err(|e| format!("Failed to serialize scene to binary: {e}"))?;
		fs::write(path, bytes).map_err(|e| format!("Failed to write binary scene {path}: {e}"))
	}

	pub fn instantiate(
		&self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		rendering: &RenderingSystem,
		scene: &mut Scene,
	) -> Result<(), String> {
		let template = rendering
			.default_template()
			.ok_or_else(|| "Default MaterialTemplate not available in RenderingSystem".to_string())?;

		for entity_desc in &self.entities {
			let transform = entity_desc
				.transform
				.as_ref()
				.map(|t| TransformComponent(t.to_transform()))
				.unwrap_or_default();

			if let Some(cam) = &entity_desc.camera {
				let mut camera_comp = CameraComponent::new(cam.fov_deg, cam.near, cam.far);
				camera_comp.is_active = cam.is_active;
				camera_comp.clear_color = Vec4::new_with(
					cam.clear_color[0],
					cam.clear_color[1],
					cam.clear_color[2],
					cam.clear_color[3],
				);
				scene.spawn((transform, camera_comp));
			}

			if let Some(dl) = &entity_desc.directional_light {
				let dir_light = DirectionalLightComponent {
					color: Vec3::new(dl.color[0], dl.color[1], dl.color[2]),
					intensity: dl.intensity,
					direction: Vec3::new(dl.direction[0], dl.direction[1], dl.direction[2]).normalize(),
					cast_shadows: dl.cast_shadows,
				};
				scene.spawn((transform, dir_light));
			}

			if let Some(pl) = &entity_desc.point_light {
				let pt_light = PointLightComponent {
					color: Vec3::new(pl.color[0], pl.color[1], pl.color[2]),
					intensity: pl.intensity,
					radius: pl.radius,
				};
				scene.spawn((transform, pt_light));
			}

			if let Some(mr) = &entity_desc.mesh_renderer {
				match ModelAsset::load(&mr.asset) {
					Ok(model) => {
						let submeshes = model.build_runtime(device, alloc, &template);
						for (mesh, mat) in submeshes {
							let mut renderer = MeshRendererComponent::new(mesh, mat);
							renderer.is_visible = mr.is_visible;
							renderer.cast_shadows = mr.cast_shadows;
							scene.spawn((transform, renderer));
						}
					}
					Err(err) => {
						mochi_error!("Failed to load model asset '{}': {err}", mr.asset);
					}
				}
			}
		}

		Ok(())
	}
}
