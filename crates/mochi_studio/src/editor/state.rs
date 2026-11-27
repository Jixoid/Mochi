/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};

use mochi_asset::{
	CameraDesc, ColliderDesc, DirectionalLightDesc, EntityDesc, MeshRendererDesc, PointLightDesc,
	RigidBodyDesc, SceneAsset, SpotLightDesc, TransformDesc,
};
use nimbus::core::{ObjectID, UiTree};
use nimbus::math::Color;
use nimbus::theme::{DColor, MonetRole, Theme};

use crate::projects::{LaunchOptions, SceneEntry};
use crate::scene_overlays::ViewportOverlayOptions;
use crate::viewport_renderer::ViewportController;


const MAX_HISTORY_LEN: usize = 64;


#[derive(Clone, Copy)]
pub struct EditorIds {
	pub scene_title: ObjectID,
	pub hierarchy_tree: ObjectID,
	pub inspector_content: ObjectID,
	pub asset_browser_list: ObjectID,
	pub console_log_list: ObjectID,
	pub main_viewport: ObjectID,
	pub transform_gizmo: ObjectID,
	pub orientation_gizmo: ObjectID,
	pub viewport_loading: ObjectID,
	pub viewport_hud: ObjectID,
	pub viewport_camera_hud: ObjectID,
	pub status_text: ObjectID,
	pub viewport_size_text: ObjectID,
}


pub struct OrbitCameraState {
	pub yaw_deg: f32,
	pub pitch_deg: f32,
	pub distance: f32,
	pub target: [f32; 3],
	pub dragging: bool,
	pub last_x: f32,
	pub last_y: f32,
}

impl Default for OrbitCameraState {
	fn default() -> Self {
		Self {
			yaw_deg: 25.0,
			pitch_deg: 20.0,
			distance: 10.0,
			target: [0.0, 0.0, 0.0],
			dragging: false,
			last_x: 0.0,
			last_y: 0.0,
		}
	}
}


#[derive(Clone)]
pub struct HistorySnapshot {
	pub scene: SceneAsset,
	pub selected_index: Option<usize>,
	pub label: String,
}

#[derive(Default)]
pub struct HistoryStack {
	pub undo: Vec<HistorySnapshot>,
	pub redo: Vec<HistorySnapshot>,
}

impl HistoryStack {
	pub fn push_snapshot(
		&mut self,
		scene: &SceneAsset,
		selected_index: Option<usize>,
		label: impl Into<String>,
	) {
		if self.undo.len() >= MAX_HISTORY_LEN {
			self.undo.remove(0);
		}
		self.undo.push(HistorySnapshot {
			scene: scene.clone(),
			selected_index,
			label: label.into(),
		});
		self.redo.clear();
	}
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityPreset {
	Empty,
	DirectionalLight,
	PointLight,
	SpotLight,
	Camera,
	PhysicsCube,
}

impl EntityPreset {
	pub fn build_entity(self, index: usize) -> EntityDesc {
		match self {
			Self::Empty => EntityDesc {
				name: Some(format!("Entity_{index:02}")),
				transform: Some(TransformDesc::default()),
				..Default::default()
			},
			Self::DirectionalLight => EntityDesc {
				name: Some(format!("DirectionalLight_{index:02}")),
				transform: Some(TransformDesc {
					translation: [0.0, 6.0, 0.0],
					..Default::default()
				}),
				directional_light: Some(DirectionalLightDesc {
					color: [1.0, 0.96, 0.90],
					intensity: 2.5,
					direction: [0.4, -1.0, 0.3],
					cast_shadows: true,
				}),
				..Default::default()
			},
			Self::PointLight => EntityDesc {
				name: Some(format!("PointLight_{index:02}")),
				transform: Some(TransformDesc {
					translation: [0.0, 3.0, 0.0],
					..Default::default()
				}),
				point_light: Some(PointLightDesc {
					color: [1.0, 0.85, 0.65],
					intensity: 4.0,
					radius: 12.0,
				}),
				..Default::default()
			},
			Self::SpotLight => EntityDesc {
				name: Some(format!("SpotLight_{index:02}")),
				transform: Some(TransformDesc {
					translation: [0.0, 4.0, 2.0],
					..Default::default()
				}),
				spot_light: Some(SpotLightDesc {
					color: [1.0, 0.95, 0.85],
					intensity: 6.0,
					radius: 18.0,
					inner_cone_angle_deg: 20.0,
					outer_cone_angle_deg: 35.0,
					cast_shadows: true,
				}),
				..Default::default()
			},
			Self::Camera => EntityDesc {
				name: Some(format!("Camera_{index:02}")),
				transform: Some(TransformDesc {
					translation: [0.0, 2.0, 6.0],
					..Default::default()
				}),
				camera: Some(CameraDesc {
					is_active: false,
					..Default::default()
				}),
				..Default::default()
			},
			Self::PhysicsCube => EntityDesc {
				name: Some(format!("PhysicsCube_{index:02}")),
				transform: Some(TransformDesc {
					translation: [0.0, 3.0, 0.0],
					..Default::default()
				}),
				mesh_renderer: Some(MeshRendererDesc {
					asset: "cube".to_string(),
					is_visible: true,
					cast_shadows: true,
				}),
				rigid_body: Some(RigidBodyDesc::default()),
				collider: Some(ColliderDesc::default()),
				..Default::default()
			},
		}
	}
}


pub struct EditorState {
	pub ids: Option<EditorIds>,
	pub entry: SceneEntry,
	pub scene: SceneAsset,
	pub selected_index: Option<usize>,
	pub dirty: bool,
	pub viewport_size: (u32, u32),
	pub camera: OrbitCameraState,
	pub theme: Theme,
	pub launch_options: Arc<Mutex<LaunchOptions>>,
	pub vp_ctrl: ViewportController,
	pub history: HistoryStack,
	pub add_component_choice: usize,
	pub overlays: ViewportOverlayOptions,
}

impl EditorState {
	pub fn record_undo(&mut self, label: impl Into<String>) {
		self
			.history
			.push_snapshot(&self.scene, self.selected_index, label);
	}

	pub fn reload_viewport_scene(&self) {
		self.vp_ctrl.load_scene(
			self.scene.clone(),
			self.entry.scene_path.parent().map(|p| p.to_path_buf()),
			Some(self.entry.project_dir.clone()),
		);
	}
}


pub fn clear_children(tree: &mut UiTree, parent_id: ObjectID) {
	let children = tree
		.nodes
		.get(parent_id)
		.map(|n| n.children.clone())
		.unwrap_or_default();
	for child_id in children {
		tree.remove(child_id);
	}
}


pub fn entity_display_name(entity: &EntityDesc, index: usize) -> String {
	entity
		.name
		.as_deref()
		.filter(|s| !s.trim().is_empty())
		.map(|s| s.to_string())
		.unwrap_or_else(|| format!("Entity #{index}"))
}


pub fn entity_badge_and_color(entity: &EntityDesc) -> (&'static str, DColor) {
	if entity.camera.is_some() {
		("Cam", DColor::from(Color::new(0.73, 0.53, 0.98, 1.0)))
	} else if entity.directional_light.is_some()
		|| entity.point_light.is_some()
		|| entity.spot_light.is_some()
	{
		("Light", DColor::from(Color::new(0.95, 0.79, 0.30, 1.0)))
	} else if entity.rigid_body.is_some() {
		("Phys", DColor::from(Color::new(0.32, 0.72, 0.41, 1.0)))
	} else if entity.mesh_renderer.is_some() {
		("Mesh", DColor::from(Color::new(0.39, 0.66, 0.94, 1.0)))
	} else if entity.script.is_some() {
		("Script", DColor::from(Color::new(0.94, 0.56, 0.32, 1.0)))
	} else {
		("Node", DColor::from(MonetRole::OnSurfaceVariant))
	}
}
