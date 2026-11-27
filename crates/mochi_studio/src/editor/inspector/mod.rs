/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


mod camera;
pub mod fields;
mod lights;
mod mesh;
mod physics;
mod script;
mod transform;

use std::sync::{Arc, Mutex};

use mochi_asset::{
	CameraDesc, ColliderDesc, DirectionalLightDesc, EntityDesc, MeshRendererDesc, PointLightDesc,
	RigidBodyDesc, ScriptDesc, SpotLightDesc, SpringJointDesc,
};
use nimbus::core::{Anchor, ObjectID, TaskQueue, UiTree};
use nimbus::theme::{DColor, MonetRole};
use nimbus::ui::{Button, ButtonVariant, ComboBox, Edit, Text};

use super::commands::duplicate_selected_entity;
use super::console::push_console_log;
use super::hierarchy::rebuild_hierarchy;
use super::state::{EditorState, clear_children, entity_display_name};
use super::viewport::{sync_gizmo_from_selection, update_header_and_status};
use fields::insert_section;


#[derive(Clone, Copy)]
enum AddableComponent {
	Camera,
	DirectionalLight,
	PointLight,
	SpotLight,
	MeshRenderer,
	Script,
	RigidBody,
	Collider,
	SpringJoint,
}

impl AddableComponent {
	const ALL: [Self; 9] = [
		Self::MeshRenderer,
		Self::DirectionalLight,
		Self::PointLight,
		Self::SpotLight,
		Self::Camera,
		Self::RigidBody,
		Self::Collider,
		Self::SpringJoint,
		Self::Script,
	];

	fn label(self) -> &'static str {
		match self {
			Self::Camera => "Kamera",
			Self::DirectionalLight => "Directional Light",
			Self::PointLight => "Point Light",
			Self::SpotLight => "Spot Light",
			Self::MeshRenderer => "Mesh Renderer",
			Self::Script => "Script",
			Self::RigidBody => "RigidBody",
			Self::Collider => "Collider",
			Self::SpringJoint => "Spring Joint",
		}
	}

	fn is_present(self, e: &EntityDesc) -> bool {
		match self {
			Self::Camera => e.camera.is_some(),
			Self::DirectionalLight => e.directional_light.is_some(),
			Self::PointLight => e.point_light.is_some(),
			Self::SpotLight => e.spot_light.is_some(),
			Self::MeshRenderer => e.mesh_renderer.is_some(),
			Self::Script => e.script.is_some(),
			Self::RigidBody => e.rigid_body.is_some(),
			Self::Collider => e.collider.is_some(),
			Self::SpringJoint => e.spring_joint.is_some(),
		}
	}

	fn needs_viewport_reload(self) -> bool {
		matches!(
			self,
			Self::MeshRenderer | Self::DirectionalLight | Self::PointLight | Self::SpotLight
		)
	}

	fn attach_default(self, e: &mut EntityDesc) {
		match self {
			Self::Camera => e.camera = Some(CameraDesc::default()),
			Self::DirectionalLight => {
				e.directional_light = Some(DirectionalLightDesc {
					color: [1.0, 0.96, 0.90],
					intensity: 2.0,
					direction: [0.4, -1.0, 0.3],
					cast_shadows: true,
				});
			}
			Self::PointLight => {
				e.point_light = Some(PointLightDesc {
					color: [1.0, 0.9, 0.75],
					intensity: 3.0,
					radius: 10.0,
				});
			}
			Self::SpotLight => {
				e.spot_light = Some(SpotLightDesc {
					color: [1.0, 0.95, 0.85],
					intensity: 5.0,
					radius: 15.0,
					inner_cone_angle_deg: 20.0,
					outer_cone_angle_deg: 35.0,
					cast_shadows: true,
				});
			}
			Self::MeshRenderer => {
				e.mesh_renderer = Some(MeshRendererDesc {
					asset: "cube".to_string(),
					is_visible: true,
					cast_shadows: true,
				});
			}
			Self::Script => {
				e.script = Some(ScriptDesc {
					path: "scripts/main.rhai".to_string(),
					is_active: true,
				});
			}
			Self::RigidBody => e.rigid_body = Some(RigidBodyDesc::default()),
			Self::Collider => e.collider = Some(ColliderDesc::default()),
			Self::SpringJoint => e.spring_joint = Some(SpringJointDesc::default()),
		}
	}
}


pub fn rebuild_inspector(tree: &mut UiTree, state: &Arc<Mutex<EditorState>>, tasks: &TaskQueue) {
	let (ids, selected_opt) = {
		let guard = state.lock().unwrap();
		let Some(ids) = guard.ids else { return };
		let selected_opt = guard
			.selected_index
			.and_then(|i| guard.scene.entities.get(i).cloned().map(|e| (i, e)));
		(ids, selected_opt)
	};

	clear_children(tree, ids.inspector_content);

	let Some((idx, entity)) = selected_opt else {
		tree.insert_child(
			ids.inspector_content,
			Text::builder()
				.text("İncelemek için Hiyerarşi'den bir nesne seçin.")
				.font_size(13.0)
				.color(DColor::from(MonetRole::OnSurfaceVariant))
				.build(),
		);
		return;
	};

	// 1. General (Name + Duplicate)
	let gen_sec = insert_section(tree, ids.inspector_content, "Genel", true);
	{
		let state_clone = Arc::clone(state);
		let tasks_clone = tasks.clone();
		let edit_id = tree.insert_child(
			gen_sec,
			Edit::builder()
				.title("İsim")
				.text(entity_display_name(&entity, idx))
				.on_changed(move |new_name: String| {
					{
						let mut guard = state_clone.lock().unwrap();
						if let Some(e) = guard.scene.entities.get_mut(idx) {
							e.name = Some(new_name);
							guard.dirty = true;
						}
					}
					let state_inner = Arc::clone(&state_clone);
					tasks_clone.push_tree(move |tree| {
						update_header_and_status(tree, &state_inner, None);
						rebuild_hierarchy(tree, &state_inner);
					});
				})
				.build(),
		);
		tree.set_anchor(edit_id, Some(Anchor::TopFill));

		let state_dup = Arc::clone(state);
		let tasks_dup = tasks.clone();
		let dup_id = tree.insert_child(
			gen_sec,
			Button::builder()
				.text("Nesneyi Kopyala")
				.font_size(12.0)
				.variant(ButtonVariant::Outlined)
				.on_click(move || {
					duplicate_selected_entity(&state_dup, &tasks_dup);
				})
				.build(),
		);
		tree.set_anchor(dup_id, Some(Anchor::TopFill));
	}

	// 2. Modular Component Sections
	transform::build_transform_section(tree, ids.inspector_content, &entity, state, tasks);
	camera::build_camera_section(tree, ids.inspector_content, &entity, state, tasks);
	lights::build_lights_sections(tree, ids.inspector_content, &entity, state, tasks);
	mesh::build_mesh_section(tree, ids.inspector_content, &entity, state, tasks);
	script::build_script_section(tree, ids.inspector_content, &entity, state, tasks);
	physics::build_physics_sections(tree, ids.inspector_content, &entity, state, tasks);

	// 3. "+ Bileşen Ekle" section for missing components
	build_add_component_section(tree, ids.inspector_content, &entity, state, tasks);

	tree.needs_layout = true;
	tree.needs_redraw = true;
}


fn build_add_component_section(
	tree: &mut UiTree,
	parent: ObjectID,
	entity: &EntityDesc,
	state: &Arc<Mutex<EditorState>>,
	tasks: &TaskQueue,
) {
	let missing: Vec<AddableComponent> = AddableComponent::ALL
		.into_iter()
		.filter(|c| !c.is_present(entity))
		.collect();

	if missing.is_empty() {
		return;
	}

	let labels: Vec<&str> = missing.iter().map(|c| c.label()).collect();
	let selected = {
		let mut guard = state.lock().unwrap();
		if guard.add_component_choice >= missing.len() {
			guard.add_component_choice = 0;
		}
		guard.add_component_choice
	};

	let sec = insert_section(tree, parent, "+ Bileşen Ekle", false);

	let state_sel = Arc::clone(state);
	let cb_id = tree.insert_child(
		sec,
		ComboBox::builder()
			.title("Bileşen Türü")
			.items(labels)
			.selected(selected)
			.on_changed(move |sel: usize, _label: String| {
				state_sel.lock().unwrap().add_component_choice = sel;
			})
			.build(),
	);
	tree.set_anchor(cb_id, Some(Anchor::TopFill));

	let state_add = Arc::clone(state);
	let tasks_add = tasks.clone();
	let add_btn = tree.insert_child(
		sec,
		Button::builder()
			.text("+ Seçili Bileşeni Ekle")
			.variant(ButtonVariant::Tonal)
			.on_click(move || {
				let added_label = {
					let mut guard = state_add.lock().unwrap();
					let Some(idx) = guard.selected_index else {
						return;
					};
					let choice = guard.add_component_choice.min(missing.len() - 1);
					let comp = missing[choice];
					guard.record_undo(format!("{} Ekle", comp.label()));
					if let Some(e) = guard.scene.entities.get_mut(idx) {
						comp.attach_default(e);
						guard.dirty = true;
					}
					if comp.needs_viewport_reload() {
						guard.reload_viewport_scene();
					}
					comp.label()
				};

				let state_inner = Arc::clone(&state_add);
				let tasks_inner = tasks_add.clone();
				tasks_add.push_tree(move |tree| {
					let msg = format!("{added_label} bileşeni eklendi");
					push_console_log(tree, &state_inner, "ECS", &msg);
					update_header_and_status(tree, &state_inner, Some(msg));
					rebuild_hierarchy(tree, &state_inner);
					rebuild_inspector(tree, &state_inner, &tasks_inner);
					sync_gizmo_from_selection(tree, &state_inner);
				});
			})
			.build(),
	);
	tree.set_anchor(add_btn, Some(Anchor::TopFill));
}
