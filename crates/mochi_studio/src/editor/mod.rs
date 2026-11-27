/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod asset_browser;
pub mod commands;
pub mod console;
pub mod hierarchy;
pub mod inspector;
pub mod state;
pub mod viewport;

use std::sync::{Arc, Mutex};

use mochi_asset::{SceneAsset, TransformDesc};
use nimbus::core::{MethodTable, TaskQueue, UiTree};
use nimbus::math::Color;
use nimbus::theme::{Monet, RadiusTokens, Theme};
use nimbus::ui::{Toast, ToastKind, TreeView};
use nimbus::ext::gizmo::{CameraViewPreset, GizmoMode, GizmoSpace, TransformGizmo};

use crate::projects::{self, LaunchOptions, SceneEntry};
use crate::scene_overlays::ViewportOverlayOptions;
use crate::viewport_renderer::ViewportController;

use asset_browser::populate_asset_browser;
use commands::{
	add_entity_preset, delete_selected_entity, duplicate_selected_entity, redo, save_scene, undo,
};
use console::push_console_log;
use hierarchy::rebuild_hierarchy;
use inspector::rebuild_inspector;
use state::{EditorIds, EditorState, EntityPreset, HistoryStack, OrbitCameraState};
use viewport::{
	focus_selected_entity, sync_gizmo_camera, sync_gizmo_from_selection, update_header_and_status,
	update_viewport_hud,
};


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
		history: HistoryStack::default(),
		add_component_choice: 0,
		overlays: ViewportOverlayOptions::default(),
	}));

	let methods = build_editor_methods(Arc::clone(&state), tasks.clone(), app_tasks, on_back);

	clear_tree_roots(tree);
	let qdl_scene = match nimbus::qdl::load_into_tree(
		include_str!("../../design/editor.qdl"),
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
		vp_ctrl.set_overlays(guard.overlays);
		vp_ctrl.set_selection(guard.selected_index);
		vp_ctrl.update_camera(
			guard.camera.yaw_deg,
			guard.camera.pitch_deg,
			guard.camera.distance,
			guard.camera.target,
		);
		guard.reload_viewport_scene();
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

	// Toolbar callbacks
	{
		let cb = Arc::clone(&do_back);
		methods.register("EditorBackToHub", move || cb());
	}
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("EditorSaveScene", move || save_scene(&state, &tasks));
	}
	{
		let cb = Arc::clone(&do_play);
		methods.register("EditorPlayScene", move || cb());
	}
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("EditorAddEntity", move || {
			add_entity_preset(&state, &tasks, EntityPreset::Empty);
		});
	}
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("EditorDeleteEntity", move || {
			delete_selected_entity(&state, &tasks);
		});
	}

	// MenuBar actions
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		let app_tasks = app_tasks.clone();
		methods.register(
			"OnMenuAction",
			move |action: String, _menu: String| match action.as_str() {
				"FileSave" => save_scene(&state, &tasks),
				"FileBackToHub" => do_back(),
				"EditUndo" => undo(&state, &tasks),
				"EditRedo" => redo(&state, &tasks),
				"EditAddEntity" | "CreateEmpty" => {
					add_entity_preset(&state, &tasks, EntityPreset::Empty);
				}
				"CreateDirLight" => {
					add_entity_preset(&state, &tasks, EntityPreset::DirectionalLight);
				}
				"CreatePointLight" => {
					add_entity_preset(&state, &tasks, EntityPreset::PointLight);
				}
				"CreateSpotLight" => {
					add_entity_preset(&state, &tasks, EntityPreset::SpotLight);
				}
				"CreateCamera" => {
					add_entity_preset(&state, &tasks, EntityPreset::Camera);
				}
				"CreatePhysicsCube" => {
					add_entity_preset(&state, &tasks, EntityPreset::PhysicsCube);
				}
				"EditDuplicateEntity" => duplicate_selected_entity(&state, &tasks),
				"EditDeleteEntity" => delete_selected_entity(&state, &tasks),
				"ViewFocusSelected" => {
					let state_inner = Arc::clone(&state);
					tasks.push_tree(move |tree| {
						focus_selected_entity(tree, &state_inner);
					});
				}
				"ViewToggleGrid" => {
					let enabled = {
						let mut guard = state.lock().unwrap();
						guard.overlays.show_grid = !guard.overlays.show_grid;
						let opts = guard.overlays;
						guard.vp_ctrl.set_overlays(opts);
						opts.show_grid
					};
					let state_log = Arc::clone(&state);
					tasks.push_tree(move |tree| {
						let msg = if enabled { "Izgara açıldı" } else { "Izgara gizlendi" };
						push_console_log(tree, &state_log, "UI", msg);
					});
				}
				"ViewToggleGizmos" => {
					let enabled = {
						let mut guard = state.lock().unwrap();
						guard.overlays.show_gizmos = !guard.overlays.show_gizmos;
						let opts = guard.overlays;
						guard.vp_ctrl.set_overlays(opts);
						opts.show_gizmos
					};
					let state_log = Arc::clone(&state);
					tasks.push_tree(move |tree| {
						let msg = if enabled {
							"Işık & kamera çizimleri açıldı"
						} else {
							"Işık & kamera çizimleri gizlendi"
						};
						push_console_log(tree, &state_log, "UI", msg);
					});
				}
				"ViewToggleColliders" => {
					let enabled = {
						let mut guard = state.lock().unwrap();
						guard.overlays.show_colliders = !guard.overlays.show_colliders;
						let opts = guard.overlays;
						guard.vp_ctrl.set_overlays(opts);
						opts.show_colliders
					};
					let state_log = Arc::clone(&state);
					tasks.push_tree(move |tree| {
						let msg = if enabled {
							"Tüm collider çizimleri açıldı"
						} else {
							"Yalnızca seçili collider çiziliyor"
						};
						push_console_log(tree, &state_log, "UI", msg);
					});
				}
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
						push_console_log(
							tree,
							&state_log,
							"UI",
							&format!("Tema değiştirildi: {label}"),
						);
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
			},
		);
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
					guard.vp_ctrl.set_selection(Some(idx));
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

	// Viewport resize & orbit camera
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
					let (yaw, pitch, dist, target) = (
						guard.camera.yaw_deg,
						guard.camera.pitch_deg,
						guard.camera.distance,
						guard.camera.target,
					);
					guard.vp_ctrl.update_camera(yaw, pitch, dist, target);
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
				let (yaw, pitch, dist, target) = (
					guard.camera.yaw_deg,
					guard.camera.pitch_deg,
					guard.camera.distance,
					guard.camera.target,
				);
				guard.vp_ctrl.update_camera(yaw, pitch, dist, target);
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

	// Gizmo 3D manipulation -> live in-memory ECS delta update
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

	// OrientationGizmo snap presets & direct drag orbit
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
				let (yaw, pitch, dist, target) = (
					guard.camera.yaw_deg,
					guard.camera.pitch_deg,
					guard.camera.distance,
					guard.camera.target,
				);
				guard.vp_ctrl.update_camera(yaw, pitch, dist, target);
			}
			let state = Arc::clone(&state);
			tasks.push_tree(move |tree| {
				sync_gizmo_camera(tree, &state, true);
				update_viewport_hud(tree, &state);
			});
		});
	}
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("GizmoCameraOrbit", move |dyaw: f32, dpitch: f32| {
			{
				let mut guard = state.lock().unwrap();
				guard.camera.yaw_deg = (guard.camera.yaw_deg + dyaw.to_degrees()) % 360.0;
				guard.camera.pitch_deg = (guard.camera.pitch_deg + dpitch.to_degrees()).clamp(-89.0, 89.0);
				let (yaw, pitch, dist, target) = (
					guard.camera.yaw_deg,
					guard.camera.pitch_deg,
					guard.camera.distance,
					guard.camera.target,
				);
				guard.vp_ctrl.update_camera(yaw, pitch, dist, target);
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
