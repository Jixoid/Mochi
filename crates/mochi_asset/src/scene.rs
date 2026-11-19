/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};
use crate::model::ModelAsset;
use mochi_rendering::RenderingSystem;
use mochi_rhi as rhi;
use mochi_scene::{
	CameraComponent, DirectionalLightComponent, MeshRendererComponent, PointLightComponent, Scene,
	SkeletonComponent, SpotLightComponent, TransformComponent,
};
use mochi_script::ScriptComponent;
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

fn default_translation() -> [f32; 3] {
	[0.0, 0.0, 0.0]
}
fn default_rotation() -> [f32; 4] {
	[0.0, 0.0, 0.0, 1.0]
}
fn default_scale() -> [f32; 3] {
	[1.0, 1.0, 1.0]
}

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
			translation: Vec3::new(
				self.translation[0],
				self.translation[1],
				self.translation[2],
			),
			rotation: Quat::new(
				self.rotation[0],
				self.rotation[1],
				self.rotation[2],
				self.rotation[3],
			),
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

fn default_fov() -> f32 {
	60.0
}
fn default_near() -> f32 {
	0.1
}
fn default_far() -> f32 {
	1000.0
}
fn default_active() -> bool {
	true
}
fn default_clear_color() -> [f32; 4] {
	[0.1, 0.15, 0.25, 1.0]
}

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

fn default_cast_shadows() -> bool {
	true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointLightDesc {
	pub color: [f32; 3],
	pub intensity: f32,
	pub radius: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpotLightDesc {
	pub color: [f32; 3],
	pub intensity: f32,
	pub radius: f32,
	pub inner_cone_angle_deg: f32,
	pub outer_cone_angle_deg: f32,
	#[serde(default = "default_cast_shadows")]
	pub cast_shadows: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeshRendererDesc {
	pub asset: String,
	#[serde(default = "default_visible")]
	pub is_visible: bool,
	#[serde(default = "default_cast_shadows")]
	pub cast_shadows: bool,
}

fn default_visible() -> bool {
	true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScriptDesc {
	pub path: String,
	#[serde(default = "default_active")]
	pub is_active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RigidBodyDesc {
	#[serde(default = "default_body_type")]
	pub body_type: String,
	#[serde(default = "default_mass")]
	pub mass: f32,
	#[serde(default = "default_linear_damping")]
	pub linear_damping: f32,
	#[serde(default = "default_angular_damping")]
	pub angular_damping: f32,
	#[serde(default = "default_ccd")]
	pub use_ccd: bool,
}

fn default_body_type() -> String {
	"dynamic".into()
}
fn default_mass() -> f32 {
	1.0
}
fn default_linear_damping() -> f32 {
	0.05
}
fn default_angular_damping() -> f32 {
	0.1
}
fn default_ccd() -> bool {
	false
}

impl Default for RigidBodyDesc {
	fn default() -> Self {
		Self {
			body_type: default_body_type(),
			mass: default_mass(),
			linear_damping: default_linear_damping(),
			angular_damping: default_angular_damping(),
			use_ccd: default_ccd(),
		}
	}
}

impl RigidBodyDesc {
	pub fn to_component(&self, transform: &Transform<f32>) -> mochi_scene::RigidBodyComponent {
		let b_type = match self.body_type.to_lowercase().as_str() {
			"static" => mochi_scene::BodyType::Static,
			"kinematic" => mochi_scene::BodyType::Kinematic,
			_ => mochi_scene::BodyType::Dynamic,
		};
		let def = mochi_scene::BodyDef {
			body_type: b_type,
			mass: if b_type == mochi_scene::BodyType::Static {
				0.0
			} else {
				self.mass
			},
			linear_damping: self.linear_damping,
			angular_damping: self.angular_damping,
			use_ccd: self.use_ccd,
			position: transform.translation,
			rotation: transform.rotation,
			..Default::default()
		};
		mochi_scene::RigidBodyComponent::new(def)
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColliderDesc {
	#[serde(default = "default_shape_name")]
	pub shape: String,
	#[serde(default)]
	pub half_extents: Option<[f32; 3]>,
	#[serde(default)]
	pub radius: Option<f32>,
	#[serde(default)]
	pub height: Option<f32>,
	#[serde(default = "default_friction")]
	pub friction: f32,
	#[serde(default = "default_restitution")]
	pub restitution: f32,
}

fn default_shape_name() -> String {
	"box".into()
}
fn default_friction() -> f32 {
	0.4
}
fn default_restitution() -> f32 {
	0.0
}

impl Default for ColliderDesc {
	fn default() -> Self {
		Self {
			shape: default_shape_name(),
			half_extents: None,
			radius: None,
			height: None,
			friction: default_friction(),
			restitution: default_restitution(),
		}
	}
}

impl ColliderDesc {
	pub fn to_component(&self) -> mochi_scene::ColliderComponent {
		let shape = match self.shape.to_lowercase().as_str() {
			"sphere" => {
				let r = self.radius.unwrap_or(0.5);
				mochi_scene::ColliderShape::sphere(r)
			}
			"capsule" => {
				let r = self.radius.unwrap_or(0.5);
				let h = self.height.unwrap_or(1.0);
				mochi_scene::ColliderShape::capsule(r, h)
			}
			_ => {
				let he = self.half_extents.unwrap_or([0.5, 0.5, 0.5]);
				mochi_scene::ColliderShape::box_shape(he[0], he[1], he[2])
			}
		};
		let mat = mochi_scene::PhysicsMaterial {
			friction: self.friction,
			restitution: self.restitution,
			density: 1.0,
		};
		let def = mochi_scene::ColliderDef {
			shape,
			material: mat,
			..Default::default()
		};
		mochi_scene::ColliderComponent::new(def)
	}
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EntityDesc {
	pub name: Option<String>,
	pub transform: Option<TransformDesc>,
	pub camera: Option<CameraDesc>,
	pub directional_light: Option<DirectionalLightDesc>,
	pub point_light: Option<PointLightDesc>,
	pub spot_light: Option<SpotLightDesc>,
	pub mesh_renderer: Option<MeshRendererDesc>,
	pub script: Option<ScriptDesc>,
	pub rigid_body: Option<RigidBodyDesc>,
	pub collider: Option<ColliderDesc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneAsset {
	pub name: String,
	pub entities: Vec<EntityDesc>,
}

impl SceneAsset {
	pub fn load_file(path: &std::path::Path) -> Result<Self, String> {
		let path_str = path.to_string_lossy();
		if path_str.ends_with(".mscene") || path_str.ends_with(".mscene.bin") {
			let bytes = fs::read(path)
				.map_err(|e| format!("Failed to read binary scene {}: {e}", path.display()))?;
			postcard::from_bytes::<Self>(&bytes)
				.map_err(|e| format!("Failed to deserialize binary scene {}: {e}", path.display()))
		} else {
			let content = fs::read_to_string(path)
				.map_err(|e| format!("Failed to read scene {}: {e}", path.display()))?;
			ron::from_str::<Self>(&content)
				.map_err(|e| format!("Failed to deserialize scene {}: {e}", path.display()))
		}
	}

	pub fn load(path: &str) -> Result<Self, String> {
		Self::load_file(std::path::Path::new(path))
	}

	pub fn save_ron(&self, path: &str) -> Result<(), String> {
		let s = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
			.map_err(|e| format!("Failed to serialize scene: {e}"))?;
		fs::write(path, s).map_err(|e| format!("Failed to write file {path}: {e}"))
	}

	pub fn save_binary(&self, path: &str) -> Result<(), String> {
		let bytes =
			postcard::to_stdvec(self).map_err(|e| format!("Failed to serialize scene to binary: {e}"))?;
		fs::write(path, bytes).map_err(|e| format!("Failed to write binary scene {path}: {e}"))
	}

	pub fn instantiate(
		&self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		rendering: &mut RenderingSystem,
		scene: &mut Scene,
		scene_dir: Option<&std::path::Path>,
		project_root: Option<&std::path::Path>,
	) -> Result<(), String> {
		let template = rendering
			.default_template()
			.ok_or_else(|| "Default MaterialTemplate not available in RenderingSystem".to_string())?;
		let transparent_template = rendering
			.transparent_template()
			.ok_or_else(|| "Transparent MaterialTemplate not available in RenderingSystem".to_string())?;

		type LoadedMeshInstances = Vec<(
			std::sync::Arc<mochi_scene::Mesh>,
			std::sync::Arc<mochi_material::MaterialInstance>,
		)>;

		#[derive(Clone)]
		struct LoadedModelData {
			submeshes: LoadedMeshInstances,
			joint_count: Option<usize>,
		}

		let mut loaded_models: HashMap<String, LoadedModelData> = HashMap::new();

		for entity_desc in &self.entities {
			let transform = entity_desc
				.transform
				.as_ref()
				.map(|t| TransformComponent(t.to_transform()))
				.unwrap_or_default();

			if let Some(mr) = &entity_desc.mesh_renderer {
				let resolved_model_path = crate::resolve_relative(&mr.asset, scene_dir, project_root);
				let model_key = resolved_model_path.to_string_lossy().to_string();

				let (submeshes, joint_count) = if let Some(cached) = loaded_models.get(&model_key) {
					(cached.submeshes.clone(), cached.joint_count)
				} else {
					match ModelAsset::load_file(&resolved_model_path, project_root) {
						Ok(model) => {
							let cmdmng = rendering.cmdmng().clone();
							let built = model.build_runtime(
								device,
								alloc,
								&cmdmng,
								rendering.texture_manager_mut(),
								&template,
								&transparent_template,
							);
							let jc = model.skins.first().map(|s| s.joint_node_indices.len());
							loaded_models.insert(
								model_key,
								LoadedModelData {
									submeshes: built.clone(),
									joint_count: jc,
								},
							);
							(built, jc)
						}
						Err(err) => {
							mochi_error!(
								"Failed to load model asset '{}': {err}",
								resolved_model_path.display()
							);
							(Vec::new(), None)
						}
					}
				};

				for (mesh, mat) in submeshes {
					let mut renderer = MeshRendererComponent::new(mesh, mat);
					renderer.is_visible = mr.is_visible;
					renderer.cast_shadows = mr.cast_shadows;

					let skeleton_opt = joint_count.map(SkeletonComponent::new);

					let entity = if let Some(sc) = &entity_desc.script {
						let resolved_script_path = crate::resolve_relative(&sc.path, scene_dir, project_root);
						let script_comp = ScriptComponent {
							path: resolved_script_path.to_string_lossy().to_string(),
							is_active: sc.is_active,
						};
						if let Some(skeleton) = skeleton_opt {
							scene.spawn((transform, renderer, skeleton, script_comp))
						} else {
							scene.spawn((transform, renderer, script_comp))
						}
					} else if let Some(skeleton) = skeleton_opt {
						scene.spawn((transform, renderer, skeleton))
					} else {
						scene.spawn((transform, renderer))
					};

					if let Some(rb_desc) = &entity_desc.rigid_body {
						let rb_comp = rb_desc.to_component(&transform.0);
						let _ = scene.world_mut().insert_one(entity, rb_comp);
					}
					if let Some(col_desc) = &entity_desc.collider {
						let col_comp = col_desc.to_component();
						let _ = scene.world_mut().insert_one(entity, col_comp);
					}
				}
			} else {
				let entity = scene.spawn((transform,));

				if let Some(rb_desc) = &entity_desc.rigid_body {
					let rb_comp = rb_desc.to_component(&transform.0);
					let _ = scene.world_mut().insert_one(entity, rb_comp);
				}
				if let Some(col_desc) = &entity_desc.collider {
					let col_comp = col_desc.to_component();
					let _ = scene.world_mut().insert_one(entity, col_comp);
				}

				if let Some(cam) = &entity_desc.camera {
					let mut camera_comp = CameraComponent::new(cam.fov_deg, cam.near, cam.far);
					camera_comp.is_active = cam.is_active;
					camera_comp.clear_color = Vec4::new_with(
						cam.clear_color[0],
						cam.clear_color[1],
						cam.clear_color[2],
						cam.clear_color[3],
					);
					let _ = scene.world_mut().insert_one(entity, camera_comp);
				}

				if let Some(dl) = &entity_desc.directional_light {
					let dir_light = DirectionalLightComponent {
						color: Vec3::new(dl.color[0], dl.color[1], dl.color[2]),
						intensity: dl.intensity,
						direction: Vec3::new(dl.direction[0], dl.direction[1], dl.direction[2]).normalize(),
						cast_shadows: dl.cast_shadows,
					};
					let _ = scene.world_mut().insert_one(entity, dir_light);
				}

				if let Some(pl) = &entity_desc.point_light {
					let pt_light = PointLightComponent {
						color: Vec3::new(pl.color[0], pl.color[1], pl.color[2]),
						intensity: pl.intensity,
						radius: pl.radius,
					};
					let _ = scene.world_mut().insert_one(entity, pt_light);
				}

				if let Some(sl) = &entity_desc.spot_light {
					let spot_light = SpotLightComponent {
						color: Vec3::new(sl.color[0], sl.color[1], sl.color[2]),
						intensity: sl.intensity,
						radius: sl.radius,
						inner_cone_angle_deg: sl.inner_cone_angle_deg,
						outer_cone_angle_deg: sl.outer_cone_angle_deg,
						cast_shadows: sl.cast_shadows,
					};
					let _ = scene.world_mut().insert_one(entity, spot_light);
				}

				if let Some(sc) = &entity_desc.script {
					let resolved_script_path = crate::resolve_relative(&sc.path, scene_dir, project_root);
					let script_comp = ScriptComponent {
						path: resolved_script_path.to_string_lossy().to_string(),
						is_active: sc.is_active,
					};
					let _ = scene.world_mut().insert_one(entity, script_comp);
				}
			}
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_load_main_scene_with_skinned_model() {
		let path = "../../test/khronos_test/scenes/main.mscene.ron";
		if !std::path::Path::new(path).exists() {
			return;
		}

		let scene_asset = SceneAsset::load(path).expect("Failed to deserialize main.mscene.ron");
		let has_cesium_man = scene_asset
			.entities
			.iter()
			.any(|e| e.name.as_deref() == Some("CesiumMan"));
		assert!(has_cesium_man, "Scene should contain CesiumMan entity");
	}

	#[test]
	fn test_load_physics_cubes_scene() {
		let path = "../../test/physics_test/scenes/physics_cubes.mscene.ron";
		if !std::path::Path::new(path).exists() {
			return;
		}

		let scene_asset =
			SceneAsset::load(path).expect("Failed to deserialize physics_cubes.mscene.ron");
		assert_eq!(scene_asset.name, "PhysicsCubesScene");
		let cube_count = scene_asset
			.entities
			.iter()
			.filter(|e| e.rigid_body.is_some() && e.collider.is_some())
			.count();
		assert!(
			cube_count >= 8,
			"Expected at least 8 physics entities (floor + 7 cubes), got {cube_count}"
		);
	}
}
