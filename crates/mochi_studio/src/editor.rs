/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};

use mochi_asset::{EntityDesc, SceneAsset, TransformDesc};
use nimbus::core::{Anchor, MethodTable, ObjectID, PropertyValue, TaskQueue, UiTree};
use nimbus::math::Color;
use nimbus::theme::{DColor, Monet, MonetRole, RadiusTokens, Theme};
use nimbus::ui::{
	Button, ButtonVariant, Collapsible, ComboBox, Edit, FloatSpinBox, Switch, Text, Toast,
	ToastKind, TreeItem, TreeView, VBox,
};
use nimbus_ui_gizmo::{
	CameraViewPreset, GizmoCamera, GizmoMode, GizmoSpace, OrientationGizmo, Quat as GizmoQuat,
	Rad as GizmoRad, Transform as GizmoTransform, TransformGizmo, Vec3 as GizmoVec3,
};
use nimbus_ui_inspector::Vec3Edit;

use crate::projects::{self, LaunchOptions, SceneEntry};
use crate::viewport_renderer::ViewportController;


#[derive(Clone, Copy)]
struct EditorIds {
	scene_title: ObjectID,
	hierarchy_tree: ObjectID,
	inspector_content: ObjectID,
	asset_browser_list: ObjectID,
	console_log_list: ObjectID,
	main_viewport: ObjectID,
	transform_gizmo: ObjectID,
	orientation_gizmo: ObjectID,
	viewport_loading: ObjectID,
	viewport_hud: ObjectID,
	viewport_camera_hud: ObjectID,
	status_text: ObjectID,
	viewport_size_text: ObjectID,
}

struct OrbitCameraState {
	yaw_deg: f32,
	pitch_deg: f32,
	distance: f32,
	dragging: bool,
	last_x: f32,
	last_y: f32,
}

impl Default for OrbitCameraState {
	fn default() -> Self {
		Self {
			yaw_deg: 25.0,
			pitch_deg: 20.0,
			distance: 10.0,
			dragging: false,
			last_x: 0.0,
			last_y: 0.0,
		}
	}
}

struct EditorState {
	ids: Option<EditorIds>,
	entry: SceneEntry,
	scene: SceneAsset,
	selected_index: Option<usize>,
	dirty: bool,
	viewport_size: (u32, u32),
	camera: OrbitCameraState,
	theme: Theme,
	launch_options: Arc<Mutex<LaunchOptions>>,
	vp_ctrl: ViewportController,
}


#[allow(clippy::too_many_arguments)]
pub fn open_editor(
	tree: &mut UiTree,
	entry: SceneEntry,
	launch_options: Arc<Mutex<LaunchOptions>>,
	vp_ctrl: ViewportController,
	theme: Theme,
	tasks: TaskQueue,
	app_tasks: TaskQueue,
	on_back: Arc<dyn Fn() + Send + Sync>,
) {
	let (scene, initial_status) = match SceneAsset::load_file(&entry.scene_path) {
		Ok(asset) => {
			let msg = format!(
				"Yüklendi: {} ({} nesne)",
				entry.display_path,
				asset.entities.len()
			);
			(asset, msg)
		}
		Err(err) => {
			let fallback = SceneAsset {
				name: entry.title.clone(),
				entities: Vec::new(),
			};
			(fallback, format!("Sahne yükleme hatası: {err}"))
		}
	};

	let initial_selection = if scene.entities.is_empty() {
		None
	} else {
		Some(0)
	};

	let state = Arc::new(Mutex::new(EditorState {
		ids: None,
		entry,
		scene,
		selected_index: initial_selection,
		dirty: false,
		viewport_size: (0, 0),
		camera: OrbitCameraState::default(),
		theme,
		launch_options,
		vp_ctrl: vp_ctrl.clone(),
	}));

	let methods = build_editor_methods(Arc::clone(&state), tasks.clone(), app_tasks, on_back);

	clear_tree_roots(tree);
	let qdl_scene = match nimbus::qdl::load_into_tree(
		include_str!("editor.qdl"),
		tree,
		&theme,
		&methods,
	) {
		Ok(scene) => scene,
		Err(e) => {
			eprintln!("[Mochi Studio] editor.qdl yüklenemedi: {e}");
			return;
		}
	};

	let ids = match extract_editor_ids(&qdl_scene) {
		Ok(ids) => ids,
		Err(e) => {
			eprintln!("[Mochi Studio] {e}");
			return;
		}
	};

	{
		let mut guard = state.lock().unwrap();
		guard.ids = Some(ids);

		vp_ctrl.set_viewport_node(Some(ids.main_viewport));
		vp_ctrl.set_loading_node(Some(ids.viewport_loading));
		vp_ctrl.update_camera(
			guard.camera.yaw_deg,
			guard.camera.pitch_deg,
			guard.camera.distance,
		);
		vp_ctrl.load_scene(
			guard.scene.clone(),
			guard.entry.scene_path.parent().map(|p| p.to_path_buf()),
			Some(guard.entry.project_dir.clone()),
		);
	}

	update_header_and_status(tree, &state, Some(initial_status.clone()));
	push_console_log(tree, &state, "INFO", &initial_status);
	populate_asset_browser(tree, &state);
	rebuild_hierarchy(tree, &state);
	rebuild_inspector(tree, &state, &tasks);
	sync_gizmo_camera(tree, &state, false);
	sync_gizmo_from_selection(tree, &state);
	update_viewport_hud(tree, &state);
}


fn clear_tree_roots(tree: &mut UiTree) {
	let roots = tree.roots.clone();
	for root_id in roots {
		tree.remove(root_id);
	}
	tree.needs_layout = true;
	tree.needs_redraw = true;
}


fn extract_editor_ids(scene: &nimbus::qdl::QdlScene) -> Result<EditorIds, String> {
	let get = |name: &str| {
		scene
			.find(name)
			.ok_or_else(|| format!("editor.qdl içinde '{name}' düğümü bulunamadı"))
	};

	Ok(EditorIds {
		scene_title: get("editor_scene_title")?,
		hierarchy_tree: get("hierarchy_tree")?,
		inspector_content: get("inspector_content")?,
		asset_browser_list: get("asset_browser_list")?,
		console_log_list: get("console_log_list")?,
		main_viewport: get("main_viewport")?,
		transform_gizmo: get("viewport_transform_gizmo")?,
		orientation_gizmo: get("viewport_orientation_gizmo")?,
		viewport_loading: get("viewport_loading")?,
		viewport_hud: get("viewport_hud")?,
		viewport_camera_hud: get("viewport_camera_hud")?,
		status_text: get("editor_status_text")?,
		viewport_size_text: get("editor_viewport_size")?,
	})
}


fn build_editor_methods(
	state: Arc<Mutex<EditorState>>,
	tasks: TaskQueue,
	app_tasks: TaskQueue,
	on_back: Arc<dyn Fn() + Send + Sync>,
) -> MethodTable {
	let mut methods = MethodTable::new();

	// Helper closures reused by both Toolbar buttons and MenuBar actions
	let do_save = {
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		Arc::new(move || {
			let res = {
				let mut guard = state.lock().unwrap();
				let path_str = guard.entry.scene_path.to_string_lossy().to_string();
				match guard.scene.save_ron(&path_str) {
					Ok(()) => {
						guard.dirty = false;
						Ok(format!("Kaydedildi: {}", guard.entry.display_path))
					}
					Err(e) => Err(format!("Kaydetme hatası: {e}")),
				}
			};
			let state = Arc::clone(&state);
			tasks.push_tree(move |tree| {
				let (title, msg, kind, level) = match res {
					Ok(m) => ("Sahne Kaydedildi", m, ToastKind::Success, "OK"),
					Err(m) => ("Kaydetme Hatası", m, ToastKind::Error, "ERR"),
				};
				Toast::builder()
					.title(title)
					.message(msg.clone())
					.kind(kind)
					.show(tree);
				push_console_log(tree, &state, level, &msg);
				update_header_and_status(tree, &state, Some(msg));
			});
		})
	};

	let do_back = {
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		let on_back = Arc::clone(&on_back);
		Arc::new(move || {
			state.lock().unwrap().vp_ctrl.deactivate();
			on_back();
			tasks.push_window(|win| {
				win.close();
			});
		})
	};

	let do_play = {
		let state = Arc::clone(&state);
		Arc::new(move || {
			let (entry, opts) = {
				let guard = state.lock().unwrap();
				let opts = guard.launch_options.lock().unwrap().clone();
				(guard.entry.clone(), opts)
			};
			projects::launch_scene(&entry, &opts);
		})
	};

	let do_add_entity = {
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		Arc::new(move || {
			let new_name = {
				let mut guard = state.lock().unwrap();
				let idx = guard.scene.entities.len() + 1;
				let name = format!("Entity_{idx:02}");
				let tf = TransformDesc::default();
				guard.scene.entities.push(EntityDesc {
					name: Some(name.clone()),
					transform: Some(tf.clone()),
					..Default::default()
				});
				guard.selected_index = Some(guard.scene.entities.len() - 1);
				guard.dirty = true;
				guard.vp_ctrl.add_entity(tf);
				name
			};

			let state = Arc::clone(&state);
			let tasks_inner = tasks.clone();
			tasks.push_tree(move |tree| {
				let msg = format!("Yeni nesne eklendi: {new_name}");
				push_console_log(tree, &state, "ECS", &msg);
				update_header_and_status(tree, &state, Some(msg));
				rebuild_hierarchy(tree, &state);
				rebuild_inspector(tree, &state, &tasks_inner);
				sync_gizmo_from_selection(tree, &state);
				update_viewport_hud(tree, &state);
			});
		})
	};

	let do_delete_entity = {
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		Arc::new(move || {
			let removed_name = {
				let mut guard = state.lock().unwrap();
				if let Some(idx) = guard.selected_index
					&& idx < guard.scene.entities.len()
				{
					let removed = guard.scene.entities.remove(idx);
					guard.dirty = true;
					guard.vp_ctrl.remove_entity(idx);
					guard.selected_index = if guard.scene.entities.is_empty() {
						None
					} else {
						Some(idx.min(guard.scene.entities.len() - 1))
					};
					Some(entity_display_name(&removed, idx))
				} else {
					None
				}
			};

			if let Some(name) = removed_name {
				let state = Arc::clone(&state);
				let tasks_inner = tasks.clone();
				tasks.push_tree(move |tree| {
					let msg = format!("Nesne silindi: {name}");
					push_console_log(tree, &state, "ECS", &msg);
					update_header_and_status(tree, &state, Some(msg));
					rebuild_hierarchy(tree, &state);
					rebuild_inspector(tree, &state, &tasks_inner);
					sync_gizmo_from_selection(tree, &state);
					update_viewport_hud(tree, &state);
				});
			}
		})
	};

	// Toolbar callbacks
	{
		let cb = Arc::clone(&do_back);
		methods.register("EditorBackToHub", move || cb());
	}
	{
		let cb = Arc::clone(&do_save);
		methods.register("EditorSaveScene", move || cb());
	}
	{
		let cb = Arc::clone(&do_play);
		methods.register("EditorPlayScene", move || cb());
	}
	{
		let cb = Arc::clone(&do_add_entity);
		methods.register("EditorAddEntity", move || cb());
	}
	{
		let cb = Arc::clone(&do_delete_entity);
		methods.register("EditorDeleteEntity", move || cb());
	}

	// MenuBar actions
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		let app_tasks = app_tasks.clone();
		methods.register("OnMenuAction", move |action: String, _menu: String| {
			match action.as_str() {
				"FileSave" => do_save(),
				"FileBackToHub" => do_back(),
				"EditAddEntity" => do_add_entity(),
				"EditDeleteEntity" => do_delete_entity(),
				"ScenePlay" => do_play(),
				"ViewThemeDark" | "ViewThemeLight" => {
					let dark = action == "ViewThemeDark";
					let monet = Monet::new(
						Color::from_oklch(0.675, 0.138, 65.0),
						Color::from_oklch(0.563, 0.038, 65.0),
						Color::from_oklch(0.517, 0.052, 300.0),
						dark,
					);
					state.lock().unwrap().theme.monet = monet;
					app_tasks.push_theme(move |theme| {
						theme.monet = monet;
					});
					let state_log = Arc::clone(&state);
					tasks.push_tree(move |tree| {
						let label = if dark { "Koyu" } else { "Açık" };
						push_console_log(tree, &state_log, "UI", &format!("Tema değiştirildi: {label}"));
					});
				}
				"ViewRadiusCompact" | "ViewRadiusRounded" => {
					let radius = if action == "ViewRadiusCompact" {
						RadiusTokens {
							none: 0.0,
							xsmall: 2.0,
							small: 4.0,
							medium: 6.0,
							large: 8.0,
							xlarge: 12.0,
							full: f32::MAX,
						}
					} else {
						RadiusTokens::default()
					};
					state.lock().unwrap().theme.radius = radius;
					app_tasks.push_theme(move |theme| {
						theme.radius = radius;
					});
				}
				_ => {}
			}
		});
	}

	// Dock pane closed feedback
	{
		let tasks = tasks.clone();
		methods.register("OnDockPaneClosed", move |title: String| {
			tasks.push_tree(move |tree| {
				Toast::builder()
					.title("Panel Kapatıldı")
					.message(format!("'{title}' paneli kapatıldı."))
					.kind(ToastKind::Info)
					.show(tree);
			});
		});
	}

	// Hierarchy TreeView selection & search
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("OnHierarchySelected", move |id: String, _label: String| {
			let Ok(idx) = id.parse::<usize>() else {
				return;
			};
			{
				let mut guard = state.lock().unwrap();
				if idx < guard.scene.entities.len() {
					guard.selected_index = Some(idx);
				} else {
					return;
				}
			}
			let state_inner = Arc::clone(&state);
			let tasks_inner = tasks.clone();
			tasks.push_tree(move |tree| {
				rebuild_inspector(tree, &state_inner, &tasks_inner);
				sync_gizmo_from_selection(tree, &state_inner);
			});
		});
	}
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("OnHierarchySearch", move |query: String| {
			let tv_id = state.lock().unwrap().ids.map(|i| i.hierarchy_tree);
			tasks.push_tree(move |tree| {
				if let Some(id) = tv_id
					&& let Some(tv) = tree.get_widget_mut::<TreeView>(id)
				{
					tv.filter = query;
					tree.needs_layout = true;
					tree.needs_redraw = true;
				}
			});
		});
	}

	// Viewport resize
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("ViewportResized", move |w: f32, h: f32| {
			let new_size = (w.round().max(0.0) as u32, h.round().max(0.0) as u32);
			{
				let mut guard = state.lock().unwrap();
				guard.viewport_size = new_size;
				guard.vp_ctrl.resize(new_size.0, new_size.1);
			}
			let state = Arc::clone(&state);
			tasks.push_tree(move |tree| {
				sync_gizmo_camera(tree, &state, false);
				update_viewport_hud(tree, &state);
			});
		});
	}

	// Viewport orbit camera interaction
	{
		let state = Arc::clone(&state);
		methods.register("ViewportPointerDown", move |x: f32, y: f32| {
			let mut guard = state.lock().unwrap();
			guard.camera.dragging = true;
			guard.camera.last_x = x;
			guard.camera.last_y = y;
		});
	}
	{
		let state = Arc::clone(&state);
		methods.register("ViewportPointerUp", move |_x: f32, _y: f32| {
			state.lock().unwrap().camera.dragging = false;
		});
	}
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("ViewportPointerMove", move |x: f32, y: f32| {
			let updated = {
				let mut guard = state.lock().unwrap();
				if guard.camera.dragging {
					let dx = x - guard.camera.last_x;
					let dy = y - guard.camera.last_y;
					guard.camera.last_x = x;
					guard.camera.last_y = y;
					guard.camera.yaw_deg = (guard.camera.yaw_deg + dx * 0.35) % 360.0;
					guard.camera.pitch_deg = (guard.camera.pitch_deg + dy * 0.35).clamp(-89.0, 89.0);
					let (yaw, pitch, dist) = (
						guard.camera.yaw_deg,
						guard.camera.pitch_deg,
						guard.camera.distance,
					);
					guard.vp_ctrl.update_camera(yaw, pitch, dist);
					true
				} else {
					false
				}
			};
			if updated {
				let state = Arc::clone(&state);
				tasks.push_tree(move |tree| {
					sync_gizmo_camera(tree, &state, false);
					update_viewport_hud(tree, &state);
				});
			}
		});
	}
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("ViewportScroll", move |_dx: f32, dy: f32| {
			{
				let mut guard = state.lock().unwrap();
				guard.camera.distance = (guard.camera.distance - dy * 0.5).clamp(1.0, 200.0);
				let (yaw, pitch, dist) = (
					guard.camera.yaw_deg,
					guard.camera.pitch_deg,
					guard.camera.distance,
				);
				guard.vp_ctrl.update_camera(yaw, pitch, dist);
			}
			let state = Arc::clone(&state);
			tasks.push_tree(move |tree| {
				sync_gizmo_camera(tree, &state, false);
				update_viewport_hud(tree, &state);
			});
		});
	}

	// Gizmo toolbar controls
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("GizmoModeChanged", move |idx: i32, _label: String| {
			let mode = match idx {
				1 => GizmoMode::Rotate,
				2 => GizmoMode::Scale,
				_ => GizmoMode::Translate,
			};
			let tg_id = state.lock().unwrap().ids.map(|i| i.transform_gizmo);
			tasks.push_tree(move |tree| {
				if let Some(id) = tg_id
					&& let Some(gizmo) = tree.get_widget_mut::<TransformGizmo>(id)
				{
					gizmo.mode = mode;
				}
			});
		});
	}
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("GizmoSpaceChanged", move |idx: i32, _label: String| {
			let space = match idx {
				1 => GizmoSpace::Local,
				_ => GizmoSpace::World,
			};
			let tg_id = state.lock().unwrap().ids.map(|i| i.transform_gizmo);
			tasks.push_tree(move |tree| {
				if let Some(id) = tg_id
					&& let Some(gizmo) = tree.get_widget_mut::<TransformGizmo>(id)
				{
					gizmo.space = space;
				}
			});
		});
	}
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("GizmoSnapChanged", move |enabled: bool| {
			let tg_id = state.lock().unwrap().ids.map(|i| i.transform_gizmo);
			tasks.push_tree(move |tree| {
				if let Some(id) = tg_id
					&& let Some(gizmo) = tree.get_widget_mut::<TransformGizmo>(id)
				{
					gizmo.snap.enabled = enabled;
				}
			});
		});
	}

	// Gizmo 3D manipulation -> live in-memory ECS delta update!
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("GizmoTransformChanged", move || {
			let state_inner = Arc::clone(&state);
			tasks.push_tree(move |tree| {
				let tg_id = state_inner.lock().unwrap().ids.map(|i| i.transform_gizmo);
				let Some(new_gtf) = tg_id
					.and_then(|id| tree.get_widget::<TransformGizmo>(id))
					.map(|g| g.transform)
				else {
					return;
				};

				let new_tf = TransformDesc {
					translation: [
						new_gtf.translation.x,
						new_gtf.translation.y,
						new_gtf.translation.z,
					],
					rotation: [
						new_gtf.rotation.x,
						new_gtf.rotation.y,
						new_gtf.rotation.z,
						new_gtf.rotation.w,
					],
					scale: [new_gtf.scale.x, new_gtf.scale.y, new_gtf.scale.z],
				};

				{
					let mut guard = state_inner.lock().unwrap();
					if let Some(idx) = guard.selected_index {
						if let Some(entity) = guard.scene.entities.get_mut(idx) {
							entity.transform = Some(new_tf.clone());
							guard.dirty = true;
						}
						guard.vp_ctrl.update_transform(idx, new_tf);
					}
				}
				update_header_and_status(tree, &state_inner, None);
			});
		});
	}

	// OrientationGizmo snap presets (+X / -X / +Y / -Y / +Z / -Z)
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("GizmoCameraSnap", move |preset: String| {
			let (yaw_rad, pitch_rad) = match preset.as_str() {
				"Right" => CameraViewPreset::Right.orbit_angles(),
				"Left" => CameraViewPreset::Left.orbit_angles(),
				"Top" => CameraViewPreset::Top.orbit_angles(),
				"Bottom" => CameraViewPreset::Bottom.orbit_angles(),
				"Back" => CameraViewPreset::Back.orbit_angles(),
				_ => CameraViewPreset::Front.orbit_angles(),
			};
			{
				let mut guard = state.lock().unwrap();
				guard.camera.yaw_deg = yaw_rad.to_degrees();
				guard.camera.pitch_deg = pitch_rad.to_degrees().clamp(-89.0, 89.0);
				let (yaw, pitch, dist) = (
					guard.camera.yaw_deg,
					guard.camera.pitch_deg,
					guard.camera.distance,
				);
				guard.vp_ctrl.update_camera(yaw, pitch, dist);
			}
			let state = Arc::clone(&state);
			tasks.push_tree(move |tree| {
				sync_gizmo_camera(tree, &state, true);
				update_viewport_hud(tree, &state);
			});
		});
	}

	// OrientationGizmo direct drag orbit
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("GizmoCameraOrbit", move |dyaw: f32, dpitch: f32| {
			{
				let mut guard = state.lock().unwrap();
				guard.camera.yaw_deg = (guard.camera.yaw_deg + dyaw.to_degrees()) % 360.0;
				guard.camera.pitch_deg =
					(guard.camera.pitch_deg + dpitch.to_degrees()).clamp(-89.0, 89.0);
				let (yaw, pitch, dist) = (
					guard.camera.yaw_deg,
					guard.camera.pitch_deg,
					guard.camera.distance,
				);
				guard.vp_ctrl.update_camera(yaw, pitch, dist);
			}
			let state = Arc::clone(&state);
			tasks.push_tree(move |tree| {
				sync_gizmo_camera(tree, &state, false);
				update_viewport_hud(tree, &state);
			});
		});
	}

	methods
}


fn clear_children(tree: &mut UiTree, parent_id: ObjectID) {
	let children = tree
		.nodes
		.get(parent_id)
		.map(|n| n.children.clone())
		.unwrap_or_default();
	for child_id in children {
		tree.remove(child_id);
	}
}


fn entity_display_name(entity: &EntityDesc, index: usize) -> String {
	entity
		.name
		.as_deref()
		.filter(|s| !s.trim().is_empty())
		.map(|s| s.to_string())
		.unwrap_or_else(|| format!("Entity #{index}"))
}


fn entity_badge_and_color(entity: &EntityDesc) -> (&'static str, DColor) {
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
	} else {
		("Node", DColor::from(MonetRole::OnSurfaceVariant))
	}
}


fn push_console_log(tree: &mut UiTree, state: &Arc<Mutex<EditorState>>, level: &str, msg: &str) {
	let Some(console_id) = state.lock().unwrap().ids.map(|i| i.console_log_list) else {
		return;
	};
	let color = match level {
		"OK" => DColor::from(Color::new(0.32, 0.78, 0.48, 1.0)),
		"ERR" => DColor::from(MonetRole::Error),
		"ECS" | "UI" => DColor::from(MonetRole::Primary),
		_ => DColor::from(MonetRole::OnSurfaceVariant),
	};
	let txt_id = tree.insert_child(
		console_id,
		Text::builder()
			.text(format!("[{level}] {msg}"))
			.font_size(12.0)
			.color(color)
			.build(),
	);
	tree.set_anchor(txt_id, Some(Anchor::TopFill));
	tree.needs_layout = true;
	tree.needs_redraw = true;
}


fn populate_asset_browser(tree: &mut UiTree, state: &Arc<Mutex<EditorState>>) {
	let (list_id, search_root) = {
		let guard = state.lock().unwrap();
		let Some(ids) = guard.ids else { return };
		(ids.asset_browser_list, guard.entry.project_dir.clone())
	};

	clear_children(tree, list_id);
	let scenes = projects::discover_scenes(&search_root);
	if scenes.is_empty() {
		tree.insert_child(
			list_id,
			Text::builder()
				.text("Bu projede başka sahne bulunamadı.")
				.font_size(12.0)
				.color(DColor::from(MonetRole::OnSurfaceVariant))
				.build(),
		);
		return;
	}

	for entry in scenes {
		let open_path = entry.scene_path.clone();
		let btn_id = tree.insert_child(
			list_id,
			Button::builder()
				.text(format!("{}  —  {}", entry.title, entry.display_path))
				.font_size(12.0)
				.variant(ButtonVariant::Outlined)
				.on_click(move || {
					projects::launch_editor(&open_path);
				})
				.build(),
		);
		tree.set_anchor(btn_id, Some(Anchor::TopFill));
	}
	tree.needs_layout = true;
	tree.needs_redraw = true;
}


fn update_header_and_status(
	tree: &mut UiTree,
	state: &Arc<Mutex<EditorState>>,
	status_override: Option<String>,
) {
	let (ids, title_text) = {
		let guard = state.lock().unwrap();
		let Some(ids) = guard.ids else { return };
		let star = if guard.dirty { " *" } else { "" };
		(ids, format!("{}{}", guard.scene.name, star))
	};

	if let Some(title) = tree.get_widget_mut::<Text>(ids.scene_title) {
		title.text = title_text;
	}
	if let Some(msg) = status_override
		&& let Some(status) = tree.get_widget_mut::<Text>(ids.status_text)
	{
		status.text = msg;
	}

	tree.needs_layout = true;
	tree.needs_redraw = true;
}


fn sync_gizmo_camera(tree: &mut UiTree, state: &Arc<Mutex<EditorState>>, animate: bool) {
	let (ids, w, h, yaw_deg, pitch_deg, dist) = {
		let guard = state.lock().unwrap();
		let Some(ids) = guard.ids else { return };
		(
			ids,
			guard.viewport_size.0,
			guard.viewport_size.1,
			guard.camera.yaw_deg,
			guard.camera.pitch_deg,
			guard.camera.distance,
		)
	};

	let aspect = if h > 0 {
		w as f32 / h as f32
	} else {
		16.0 / 9.0
	};
	let cam = GizmoCamera::from_orbit(
		GizmoVec3::ZERO,
		dist,
		yaw_deg.to_radians(),
		pitch_deg.to_radians(),
		GizmoRad::new(60.0f32.to_radians()),
		aspect,
	);

	if let Some(tg) = tree.get_widget_mut::<TransformGizmo>(ids.transform_gizmo) {
		if animate {
			tg.animate_camera(cam);
		} else {
			tg.set_camera(cam);
		}
	}
	if let Some(og) = tree.get_widget_mut::<OrientationGizmo>(ids.orientation_gizmo) {
		if animate {
			og.animate_camera(cam);
		} else {
			og.set_camera(cam);
		}
	}
	tree.needs_redraw = true;
}


fn sync_gizmo_from_selection(tree: &mut UiTree, state: &Arc<Mutex<EditorState>>) {
	let (ids, tf_opt) = {
		let guard = state.lock().unwrap();
		let Some(ids) = guard.ids else { return };
		let tf_opt = guard
			.selected_index
			.and_then(|idx| guard.scene.entities.get(idx))
			.map(|e| e.transform.clone().unwrap_or_default());
		(ids, tf_opt)
	};

	match tf_opt {
		Some(tf) => {
			tree.set_visible(ids.transform_gizmo, true);
			if let Some(tg) = tree.get_widget_mut::<TransformGizmo>(ids.transform_gizmo) {
				tg.transform = GizmoTransform {
					translation: GizmoVec3::new(
						tf.translation[0],
						tf.translation[1],
						tf.translation[2],
					),
					rotation: GizmoQuat::new(
						tf.rotation[0],
						tf.rotation[1],
						tf.rotation[2],
						tf.rotation[3],
					),
					scale: GizmoVec3::new(tf.scale[0], tf.scale[1], tf.scale[2]),
				};
			}
		}
		None => {
			tree.set_visible(ids.transform_gizmo, false);
		}
	}
	tree.needs_redraw = true;
}


fn update_viewport_hud(tree: &mut UiTree, state: &Arc<Mutex<EditorState>>) {
	let (ids, w, h, count, yaw, pitch, dist) = {
		let guard = state.lock().unwrap();
		let Some(ids) = guard.ids else { return };
		(
			ids,
			guard.viewport_size.0,
			guard.viewport_size.1,
			guard.scene.entities.len(),
			guard.camera.yaw_deg,
			guard.camera.pitch_deg,
			guard.camera.distance,
		)
	};

	if let Some(hud) = tree.get_widget_mut::<Text>(ids.viewport_hud) {
		hud.text = format!("Viewport • {w}x{h} • {count} nesne");
	}
	if let Some(cam_hud) = tree.get_widget_mut::<Text>(ids.viewport_camera_hud) {
		cam_hud.text = format!("Orbit: Yaw {yaw:.1}°  Pitch {pitch:.1}°  Zoom {dist:.1}");
	}
	if let Some(size_txt) = tree.get_widget_mut::<Text>(ids.viewport_size_text) {
		size_txt.text = format!("Viewport: {w}x{h}");
	}

	tree.needs_layout = true;
	tree.needs_redraw = true;
}


fn rebuild_hierarchy(tree: &mut UiTree, state: &Arc<Mutex<EditorState>>) {
	let (ids, scene_name, items, selected) = {
		let guard = state.lock().unwrap();
		let Some(ids) = guard.ids else { return };
		let items: Vec<TreeItem> = guard
			.scene
			.entities
			.iter()
			.enumerate()
			.map(|(i, e)| {
				let (badge, color) = entity_badge_and_color(e);
				TreeItem::new(entity_display_name(e, i))
					.with_id(i.to_string())
					.badge(badge)
					.color(color)
			})
			.collect();
		(ids, guard.scene.name.clone(), items, guard.selected_index)
	};

	if let Some(tv) = tree.get_widget_mut::<TreeView>(ids.hierarchy_tree) {
		let root = TreeItem::new(scene_name)
			.with_id("root")
			.expanded(true)
			.badge("Scene")
			.color(DColor::from(MonetRole::Primary))
			.children(items);
		tv.items = vec![root];
		tv.selected = selected.map(|i| vec![0, i]);
	}

	tree.needs_layout = true;
	tree.needs_redraw = true;
}


fn rebuild_inspector(tree: &mut UiTree, state: &Arc<Mutex<EditorState>>, tasks: &TaskQueue) {
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

	// General (Name)
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
	}

	// Transform (nimbus_ui_inspector Vec3Edit — color-coded scrubbable X/Y/Z)
	let transform = entity.transform.clone().unwrap_or_default();
	let tf_sec = insert_section(tree, ids.inspector_content, "Transform", true);

	{
		tree.insert_child(
			tf_sec,
			Text::builder()
				.text("Konum")
				.font_size(12.0)
				.color(DColor::from(MonetRole::OnSurfaceVariant))
				.build(),
		);
		let state_pos = Arc::clone(state);
		let tasks_pos = tasks.clone();
		let pos_id = tree.insert_child(
			tf_sec,
			Vec3Edit::new(transform.translation)
				.with_step(0.1)
				.with_on_changed(move |args: &[PropertyValue]| {
					let (Some(x), Some(y), Some(z)) = (
						args.first().and_then(|v| v.as_f32()),
						args.get(1).and_then(|v| v.as_f32()),
						args.get(2).and_then(|v| v.as_f32()),
					) else {
						return;
					};
					{
						let mut guard = state_pos.lock().unwrap();
						if let Some(sel) = guard.selected_index {
							let vp_ctrl = guard.vp_ctrl.clone();
							if let Some(e) = guard.scene.entities.get_mut(sel) {
								let tf = e.transform.get_or_insert_with(TransformDesc::default);
								tf.translation = [x, y, z];
								vp_ctrl.update_transform(sel, tf.clone());
								guard.dirty = true;
							}
						}
					}
					let state_inner = Arc::clone(&state_pos);
					tasks_pos.push_tree(move |tree| {
						update_header_and_status(tree, &state_inner, None);
						sync_gizmo_from_selection(tree, &state_inner);
					});
				}),
		);
		tree.set_anchor(pos_id, Some(Anchor::TopFill));
	}

	{
		tree.insert_child(
			tf_sec,
			Text::builder()
				.text("Ölçek")
				.font_size(12.0)
				.color(DColor::from(MonetRole::OnSurfaceVariant))
				.build(),
		);
		let state_sc = Arc::clone(state);
		let tasks_sc = tasks.clone();
		let mut sc_edit = Vec3Edit::new(transform.scale)
			.with_step(0.05)
			.with_on_changed(move |args: &[PropertyValue]| {
				let (Some(x), Some(y), Some(z)) = (
					args.first().and_then(|v| v.as_f32()),
					args.get(1).and_then(|v| v.as_f32()),
					args.get(2).and_then(|v| v.as_f32()),
				) else {
					return;
				};
				{
					let mut guard = state_sc.lock().unwrap();
					if let Some(sel) = guard.selected_index {
						let vp_ctrl = guard.vp_ctrl.clone();
						if let Some(e) = guard.scene.entities.get_mut(sel) {
							let tf = e.transform.get_or_insert_with(TransformDesc::default);
							tf.scale = [x.max(0.01), y.max(0.01), z.max(0.01)];
							vp_ctrl.update_transform(sel, tf.clone());
							guard.dirty = true;
						}
					}
				}
				let state_inner = Arc::clone(&state_sc);
				tasks_sc.push_tree(move |tree| {
					update_header_and_status(tree, &state_inner, None);
					sync_gizmo_from_selection(tree, &state_inner);
				});
			});
		sc_edit.default_values = [1.0, 1.0, 1.0];
		sc_edit.min = Some(0.01);
		let sc_id = tree.insert_child(tf_sec, sc_edit);
		tree.set_anchor(sc_id, Some(Anchor::TopFill));
	}

	// Camera (if present)
	if let Some(cam) = &entity.camera {
		let cam_sec = insert_section(tree, ids.inspector_content, "Kamera", true);
		insert_float_field(
			tree,
			cam_sec,
			"FOV",
			cam.fov_deg,
			10.0,
			160.0,
			1.0,
			"°",
			Arc::clone(state),
			tasks.clone(),
			|e, _e_idx, _vp, val| {
				if let Some(c) = &mut e.camera {
					c.fov_deg = val.clamp(10.0, 160.0);
				}
			},
		);
		let state_clone = Arc::clone(state);
		let tasks_clone = tasks.clone();
		tree.insert_child(
			cam_sec,
			Switch::builder()
				.text("Aktif Kamera")
				.checked(cam.is_active)
				.on_changed(move |checked: bool| {
					{
						let mut guard = state_clone.lock().unwrap();
						if let Some(e) = guard.scene.entities.get_mut(idx)
							&& let Some(c) = &mut e.camera
						{
							c.is_active = checked;
							guard.dirty = true;
						}
					}
					let state_inner = Arc::clone(&state_clone);
					tasks_clone.push_tree(move |tree| {
						update_header_and_status(tree, &state_inner, None);
					});
				})
				.build(),
		);
	}

	// Lights (if present)
	if let Some(light) = &entity.directional_light {
		let l_sec = insert_section(tree, ids.inspector_content, "Directional Light", true);
		insert_float_field(
			tree,
			l_sec,
			"Yoğunluk",
			light.intensity,
			0.0,
			100.0,
			0.2,
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, e_idx, vp, val| {
				if let Some(l) = &mut e.directional_light {
					l.intensity = val.max(0.0);
					vp.update_directional_light(e_idx, l.intensity);
				}
			},
		);
	}
	if let Some(light) = &entity.point_light {
		let l_sec = insert_section(tree, ids.inspector_content, "Point Light", true);
		insert_float_field(
			tree,
			l_sec,
			"Yoğunluk",
			light.intensity,
			0.0,
			100.0,
			0.2,
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, e_idx, vp, val| {
				if let Some(l) = &mut e.point_light {
					l.intensity = val.max(0.0);
					vp.update_point_light(e_idx, l.intensity, l.radius);
				}
			},
		);
		insert_float_field(
			tree,
			l_sec,
			"Yarıçap",
			light.radius,
			0.1,
			500.0,
			0.5,
			" m",
			Arc::clone(state),
			tasks.clone(),
			|e, e_idx, vp, val| {
				if let Some(l) = &mut e.point_light {
					l.radius = val.max(0.1);
					vp.update_point_light(e_idx, l.intensity, l.radius);
				}
			},
		);
	}

	// Mesh Renderer (if present)
	if let Some(mr) = &entity.mesh_renderer {
		let m_sec = insert_section(tree, ids.inspector_content, "Mesh Renderer", true);
		tree.insert_child(
			m_sec,
			Text::builder()
				.text(format!("Varlık: {}", mr.asset))
				.font_size(12.0)
				.color(DColor::from(MonetRole::OnSurfaceVariant))
				.build(),
		);

		let state_vis = Arc::clone(state);
		let tasks_vis = tasks.clone();
		tree.insert_child(
			m_sec,
			Switch::builder()
				.text("Görünür")
				.checked(mr.is_visible)
				.on_changed(move |checked: bool| {
					{
						let mut guard = state_vis.lock().unwrap();
						let updated = if let Some(e) = guard.scene.entities.get_mut(idx)
							&& let Some(m) = &mut e.mesh_renderer
						{
							m.is_visible = checked;
							Some((m.is_visible, m.cast_shadows))
						} else {
							None
						};
						if let Some((vis, sh)) = updated {
							guard.dirty = true;
							guard.vp_ctrl.update_mesh_flags(idx, vis, sh);
						}
					}
					let state_inner = Arc::clone(&state_vis);
					tasks_vis.push_tree(move |tree| {
						update_header_and_status(tree, &state_inner, None);
					});
				})
				.build(),
		);

		let state_sh = Arc::clone(state);
		let tasks_sh = tasks.clone();
		tree.insert_child(
			m_sec,
			Switch::builder()
				.text("Gölge Düşür")
				.checked(mr.cast_shadows)
				.on_changed(move |checked: bool| {
					{
						let mut guard = state_sh.lock().unwrap();
						let updated = if let Some(e) = guard.scene.entities.get_mut(idx)
							&& let Some(m) = &mut e.mesh_renderer
						{
							m.cast_shadows = checked;
							Some((m.is_visible, m.cast_shadows))
						} else {
							None
						};
						if let Some((vis, sh)) = updated {
							guard.dirty = true;
							guard.vp_ctrl.update_mesh_flags(idx, vis, sh);
						}
					}
					let state_inner = Arc::clone(&state_sh);
					tasks_sh.push_tree(move |tree| {
						update_header_and_status(tree, &state_inner, None);
					});
				})
				.build(),
		);
	}

	// Physics (Tachyon)
	if entity.rigid_body.is_some() || entity.collider.is_some() {
		let p_sec = insert_section(tree, ids.inspector_content, "Fizik", true);
		if let Some(rb) = &entity.rigid_body {
			let body_types = ["dynamic", "static", "kinematic"];
			let selected_bt = body_types
				.iter()
				.position(|t| t.eq_ignore_ascii_case(&rb.body_type))
				.unwrap_or(0);
			let state_bt = Arc::clone(state);
			let tasks_bt = tasks.clone();
			let cb_id = tree.insert_child(
				p_sec,
				ComboBox::builder()
					.title("Gövde Tipi")
					.items(body_types)
					.selected(selected_bt)
					.on_changed(move |_sel: usize, label: String| {
						{
							let mut guard = state_bt.lock().unwrap();
							if let Some(e) = guard.scene.entities.get_mut(idx)
								&& let Some(r) = &mut e.rigid_body
							{
								r.body_type = label;
								guard.dirty = true;
							}
						}
						let state_inner = Arc::clone(&state_bt);
						tasks_bt.push_tree(move |tree| {
							update_header_and_status(tree, &state_inner, None);
						});
					})
					.build(),
			);
			tree.set_anchor(cb_id, Some(Anchor::TopFill));

			insert_float_field(
				tree,
				p_sec,
				"Kütle",
				rb.mass,
				0.0,
				10_000.0,
				0.5,
				" kg",
				Arc::clone(state),
				tasks.clone(),
				|e, _e_idx, _vp, val| {
					if let Some(r) = &mut e.rigid_body {
						r.mass = val.max(0.0);
					}
				},
			);
		}
		if let Some(col) = &entity.collider {
			let shapes = ["box", "sphere", "capsule"];
			let selected_sh = shapes
				.iter()
				.position(|s| s.eq_ignore_ascii_case(&col.shape))
				.unwrap_or(0);
			let state_sh = Arc::clone(state);
			let tasks_sh = tasks.clone();
			let cb_id = tree.insert_child(
				p_sec,
				ComboBox::builder()
					.title("Collider Şekli")
					.items(shapes)
					.selected(selected_sh)
					.on_changed(move |_sel: usize, label: String| {
						{
							let mut guard = state_sh.lock().unwrap();
							if let Some(e) = guard.scene.entities.get_mut(idx)
								&& let Some(c) = &mut e.collider
							{
								c.shape = label;
								guard.dirty = true;
							}
						}
						let state_inner = Arc::clone(&state_sh);
						tasks_sh.push_tree(move |tree| {
							update_header_and_status(tree, &state_inner, None);
						});
					})
					.build(),
			);
			tree.set_anchor(cb_id, Some(Anchor::TopFill));

			insert_float_field(
				tree,
				p_sec,
				"Sürtünme",
				col.friction,
				0.0,
				2.0,
				0.05,
				"",
				Arc::clone(state),
				tasks.clone(),
				|e, _e_idx, _vp, val| {
					if let Some(c) = &mut e.collider {
						c.friction = val.clamp(0.0, 2.0);
					}
				},
			);
			insert_float_field(
				tree,
				p_sec,
				"Sekme",
				col.restitution,
				0.0,
				1.0,
				0.05,
				"",
				Arc::clone(state),
				tasks.clone(),
				|e, _e_idx, _vp, val| {
					if let Some(c) = &mut e.collider {
						c.restitution = val.clamp(0.0, 1.0);
					}
				},
			);
		}
	}

	tree.needs_layout = true;
	tree.needs_redraw = true;
}


fn insert_section(tree: &mut UiTree, parent: ObjectID, title: &str, expanded: bool) -> ObjectID {
	let col_id = tree.insert_child(
		parent,
		Collapsible::builder()
			.title(title)
			.expanded(expanded)
			.spacing(8.0)
			.paddings(10.0)
			.corner_radius(10.0)
			.build(),
	);
	tree.set_anchor(col_id, Some(Anchor::TopFill));

	let vbox_id = tree.insert_child(
		col_id,
		VBox::builder()
			.spacing(8.0)
			.color(Color::TRANSPARENT)
			.build(),
	);
	tree.set_anchor(vbox_id, Some(Anchor::TopFill));
	vbox_id
}


#[allow(clippy::too_many_arguments)]
fn insert_float_field<F>(
	tree: &mut UiTree,
	parent: ObjectID,
	label: &str,
	initial: f32,
	min: f64,
	max: f64,
	step: f64,
	suffix: &str,
	state: Arc<Mutex<EditorState>>,
	tasks: TaskQueue,
	apply: F,
) where
	F: Fn(&mut EntityDesc, usize, &ViewportController, f32) + Send + Sync + 'static,
{
	let mut builder = FloatSpinBox::builder()
		.title(label)
		.value(initial as f64)
		.range(min, max)
		.step(step)
		.decimals(2)
		.on_changed(move |val: f64| {
			let val_f32 = val as f32;
			{
				let mut guard = state.lock().unwrap();
				if let Some(idx) = guard.selected_index {
					let vp_ctrl = guard.vp_ctrl.clone();
					if let Some(entity) = guard.scene.entities.get_mut(idx) {
						apply(entity, idx, &vp_ctrl, val_f32);
						guard.dirty = true;
					}
				}
			}
			let state_inner = Arc::clone(&state);
			tasks.push_tree(move |tree| {
				update_header_and_status(tree, &state_inner, None);
				sync_gizmo_from_selection(tree, &state_inner);
			});
		});

	if !suffix.is_empty() {
		builder = builder.suffix(suffix);
	}

	let spin_id = tree.insert_child(parent, builder.build());
	tree.set_anchor(spin_id, Some(Anchor::TopFill));
}
