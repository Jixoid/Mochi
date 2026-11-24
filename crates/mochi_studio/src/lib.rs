/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod editor;
pub mod projects;
pub mod viewport_renderer;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use nimbus::core::{
	Anchor, Application, MethodTable, ObjectID, TaskQueue, UiTree, WidgetRegistry, Window,
};
use nimbus::math::Color;
use nimbus::theme::{
	ElevationTokens, Monet, MonetRole, RadiusTokens, StateTokens, StrokeTokens, Theme,
};
use nimbus::ui::{Button, ButtonVariant, Card, HBox, Spacer, Text, Toast, ToastKind, VBox};

use crate::projects::{LaunchOptions, SceneEntry};
use crate::viewport_renderer::{StudioViewportBridge, ViewportController};


#[derive(Clone, Copy, PartialEq, Eq)]
enum WelcomeTab {
	Projects,
	NewProject,
	Settings,
}

#[derive(Clone, Copy)]
struct WelcomeIds {
	nav_projects: ObjectID,
	nav_new_scene: ObjectID,
	nav_settings: ObjectID,
	page_projects: ObjectID,
	page_new_scene: ObjectID,
	page_settings: ObjectID,
	scene_list: ObjectID,
	create_status: ObjectID,
}

struct StudioState {
	ids: Option<WelcomeIds>,
	launch_options: Arc<Mutex<LaunchOptions>>,
	vp_ctrl: ViewportController,
	project_name: String,
	project_path: String,
	project_desc: String,
	project_no_git: bool,
	search_query: String,
	scene_cards: Vec<(ObjectID, SceneEntry)>,
	cwd: PathBuf,
	theme: Theme,
}

impl StudioState {
	fn new(cwd: PathBuf, theme: Theme, vp_ctrl: ViewportController) -> Self {
		Self {
			ids: None,
			launch_options: Arc::new(Mutex::new(LaunchOptions::default())),
			vp_ctrl,
			project_name: "my_mochi_game".to_string(),
			project_path: "./projects/my_mochi_game".to_string(),
			project_desc: "Mochi 3D Engine projesi".to_string(),
			project_no_git: false,
			search_query: String::new(),
			scene_cards: Vec::new(),
			cwd,
			theme,
		}
	}
}


pub fn run(initial_scene: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
	nimbus::except::start();
	{
		let mut reg = WidgetRegistry::global().write().unwrap();
		nimbus::ui::register_widgets(&mut reg);
		nimbus_ui_gizmo::register_widgets(&mut reg);
	}

	let monet = Monet::from_system().unwrap_or_else(|| {
		Monet::new(
			Color::from_oklch(0.675, 0.138, 65.0),
			Color::from_oklch(0.563, 0.038, 65.0),
			Color::from_oklch(0.517, 0.052, 300.0),
			true,
		)
	});

	let theme = Theme {
		elevation: ElevationTokens::default(),
		radius: RadiusTokens::default(),
		state: StateTokens::default(),
		stroke: StrokeTokens::default(),
		monet,
	};

	let mut app = Application::new(theme);
	let theme = app.theme;

	let window_handle = app.create_window(Window::new("Mochi Studio", 1280, 760));
	let main_window = app
		.window_mut(window_handle)
		.ok_or("Failed to get main window")?;

	let tasks = main_window.task_queue();
	let (mut vp_bridge, vp_ctrl) = StudioViewportBridge::spawn(tasks.clone());
	let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
	let state = Arc::new(Mutex::new(StudioState::new(cwd.clone(), theme, vp_ctrl.clone())));

	if let Some(scene_path) = initial_scene {
		let project_dir = scene_path
			.parent()
			.and_then(|p| {
				if p.file_name().and_then(|n| n.to_str()) == Some("scenes") {
					p.parent()
				} else {
					Some(p)
				}
			})
			.unwrap_or(&cwd)
			.to_path_buf();

		let title = scene_path
			.file_name()
			.and_then(|s| s.to_str())
			.unwrap_or("Sahne")
			.to_string();

		let display_path = scene_path.to_string_lossy().replace('\\', "/");
		let entry = SceneEntry {
			title,
			project_dir,
			scene_path,
			display_path,
		};

		let launch_options = Arc::clone(&state.lock().unwrap().launch_options);
		let back_state = Arc::clone(&state);
		let back_tasks = tasks.clone();
		editor::open_editor(
			main_window.tree_mut(),
			entry,
			launch_options,
			vp_ctrl,
			theme,
			tasks,
			Arc::new(move |tree| {
				let _ = mount_welcome(tree, &back_state, &back_tasks);
			}),
		);
	} else {
		mount_welcome(main_window.tree_mut(), &state, &tasks)?;
	}

	app.set_on_pre_render(move |device, cmd_manager, window| {
		vp_bridge.poll(device, cmd_manager, window);
	});

	app.run().map_err(|e| format!("Application error: {e}"))?;
	Ok(())
}


fn clear_tree_roots(tree: &mut UiTree) {
	let roots = tree.roots.clone();
	for root_id in roots {
		tree.remove(root_id);
	}
	tree.needs_layout = true;
	tree.needs_redraw = true;
}


fn mount_welcome(
	tree: &mut UiTree,
	state: &Arc<Mutex<StudioState>>,
	tasks: &TaskQueue,
) -> Result<(), String> {
	let theme = state.lock().unwrap().theme;
	let methods = build_method_table(Arc::clone(state), tasks.clone());

	clear_tree_roots(tree);
	let qdl_scene = nimbus::qdl::load_into_tree(include_str!("welcome.qdl"), tree, &theme, &methods)
		.map_err(|e| format!("Failed to parse welcome.qdl: {e}"))?;

	let ids = extract_welcome_ids(&qdl_scene)?;
	{
		let mut guard = state.lock().unwrap();
		guard.ids = Some(ids);
	}

	reload_scene_lists(tree, state, tasks);
	Ok(())
}


fn extract_welcome_ids(scene: &nimbus::qdl::QdlScene) -> Result<WelcomeIds, String> {
	let get = |name: &str| {
		scene
			.find(name)
			.ok_or_else(|| format!("welcome.qdl içinde '{name}' düğümü bulunamadı"))
	};

	Ok(WelcomeIds {
		nav_projects: get("nav_projects")?,
		nav_new_scene: get("nav_new_scene")?,
		nav_settings: get("nav_settings")?,
		page_projects: get("page_projects")?,
		page_new_scene: get("page_new_scene")?,
		page_settings: get("page_settings")?,
		scene_list: get("scene_list")?,
		create_status: get("create_status")?,
	})
}


fn build_method_table(state: Arc<Mutex<StudioState>>, tasks: TaskQueue) -> MethodTable {
	let mut methods = MethodTable::new();

	// Sidebar navigation
	for (name, tab) in [
		("NavProjects", WelcomeTab::Projects),
		("NavNewScene", WelcomeTab::NewProject),
		("NavSettings", WelcomeTab::Settings),
	] {
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register(name, move || {
			if let Some(ids) = state.lock().unwrap().ids {
				tasks.push_tree(move |tree| switch_tab(tree, ids, tab));
			}
		});
	}

	// Refresh scenes
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("RefreshScenes", move || {
			let state = Arc::clone(&state);
			let tasks_inner = tasks.clone();
			tasks.push_tree(move |tree| {
				reload_scene_lists(tree, &state, &tasks_inner);
			});
		});
	}

	// Live search filter
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("OnSearchChanged", move |query: String| {
			let cards: Vec<(ObjectID, bool)> = {
				let mut guard = state.lock().unwrap();
				guard.search_query = query;
				let needle = guard.search_query.trim().to_lowercase();
				guard
					.scene_cards
					.iter()
					.map(|(id, entry)| {
						let visible = needle.is_empty()
							|| entry.title.to_lowercase().contains(&needle)
							|| entry.display_path.to_lowercase().contains(&needle);
						(*id, visible)
					})
					.collect()
			};

			tasks.push_tree(move |tree| {
				for (id, visible) in cards {
					tree.set_visible(id, visible);
				}
			});
		});
	}

	// New project form bindings
	{
		let state = Arc::clone(&state);
		methods.register("OnProjectNameChanged", move |val: String| {
			state.lock().unwrap().project_name = val;
		});
	}
	{
		let state = Arc::clone(&state);
		methods.register("OnProjectPathChanged", move |val: String| {
			state.lock().unwrap().project_path = val;
		});
	}
	{
		let state = Arc::clone(&state);
		methods.register("OnProjectDescChanged", move |val: String| {
			state.lock().unwrap().project_desc = val;
		});
	}
	{
		let state = Arc::clone(&state);
		methods.register("OnNoGitChanged", move |checked: bool| {
			state.lock().unwrap().project_no_git = checked;
		});
	}
	{
		let state = Arc::clone(&state);
		let tasks = tasks.clone();
		methods.register("CreateProject", move || {
			let (path, name, desc, no_git, ids, theme) = {
				let guard = state.lock().unwrap();
				(
					guard.project_path.clone(),
					guard.project_name.clone(),
					guard.project_desc.clone(),
					guard.project_no_git,
					guard.ids,
					guard.theme,
				)
			};

			let result = projects::create_project(&path, &name, &desc, no_git);
			let state_clone = Arc::clone(&state);
			let tasks_inner = tasks.clone();

			if let Some(ids) = ids {
				tasks.push_tree(move |tree| {
					let (toast_title, toast_kind, msg, color, ok) = match result {
						Ok(ok_msg) => (
							"Proje Oluşturuldu",
							ToastKind::Success,
							ok_msg,
							theme.monet[MonetRole::Primary],
							true,
						),
						Err(err_msg) => (
							"Proje Hatası",
							ToastKind::Error,
							err_msg,
							theme.monet[MonetRole::Error],
							false,
						),
					};

					Toast::builder()
						.title(toast_title)
						.message(msg.clone())
						.kind(toast_kind)
						.show(tree);

					if let Some(status_text) = tree.get_widget_mut::<Text>(ids.create_status) {
						status_text.text = msg;
						status_text.color = Some(color);
						tree.needs_layout = true;
						tree.needs_redraw = true;
					}

					if ok {
						reload_scene_lists(tree, &state_clone, &tasks_inner);
					}
				});
			}
		});
	}

	// Settings switches
	{
		let state = Arc::clone(&state);
		methods.register("OnVsyncChanged", move |checked: bool| {
			state
				.lock()
				.unwrap()
				.launch_options
				.lock()
				.unwrap()
				.vsync = checked;
		});
	}
	{
		let state = Arc::clone(&state);
		methods.register("OnValidationChanged", move |checked: bool| {
			state
				.lock()
				.unwrap()
				.launch_options
				.lock()
				.unwrap()
				.validation = checked;
		});
	}
	{
		let state = Arc::clone(&state);
		methods.register("OnMuteChanged", move |checked: bool| {
			state
				.lock()
				.unwrap()
				.launch_options
				.lock()
				.unwrap()
				.mute = checked;
		});
	}
	{
		let state = Arc::clone(&state);
		methods.register("OnFgChanged", move |checked: bool| {
			state
				.lock()
				.unwrap()
				.launch_options
				.lock()
				.unwrap()
				.frame_gen = checked;
		});
	}

	methods
}


fn switch_tab(tree: &mut UiTree, ids: WelcomeIds, active: WelcomeTab) {
	let tabs = [
		(WelcomeTab::Projects, ids.nav_projects, ids.page_projects),
		(WelcomeTab::NewProject, ids.nav_new_scene, ids.page_new_scene),
		(WelcomeTab::Settings, ids.nav_settings, ids.page_settings),
	];

	for (tab, nav_id, page_id) in tabs {
		let is_active = tab == active;
		tree.set_visible(page_id, is_active);
		if let Some(btn) = tree.get_widget_mut::<Button>(nav_id) {
			btn.variant = if is_active {
				ButtonVariant::Tonal
			} else {
				ButtonVariant::Outlined
			};
		}
	}

	tree.needs_layout = true;
	tree.needs_redraw = true;
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


fn reload_scene_lists(
	tree: &mut UiTree,
	state: &Arc<Mutex<StudioState>>,
	tasks: &TaskQueue,
) {
	let (ids, cwd, theme, needle) = {
		let guard = state.lock().unwrap();
		match guard.ids {
			Some(ids) => (
				ids,
				guard.cwd.clone(),
				guard.theme,
				guard.search_query.trim().to_lowercase(),
			),
			None => return,
		}
	};

	let discovered = projects::discover_scenes(&cwd);

	clear_children(tree, ids.scene_list);
	let created_cards = populate_scene_cards(tree, ids.scene_list, &discovered, &theme, state, tasks);

	for (card_id, entry) in &created_cards {
		let visible = needle.is_empty()
			|| entry.title.to_lowercase().contains(&needle)
			|| entry.display_path.to_lowercase().contains(&needle);
		tree.set_visible(*card_id, visible);
	}

	state.lock().unwrap().scene_cards = created_cards;
	tree.needs_layout = true;
	tree.needs_redraw = true;
}


fn populate_scene_cards(
	tree: &mut UiTree,
	list_id: ObjectID,
	entries: &[SceneEntry],
	theme: &Theme,
	state: &Arc<Mutex<StudioState>>,
	tasks: &TaskQueue,
) -> Vec<(ObjectID, SceneEntry)> {
	let mut cards = Vec::with_capacity(entries.len());

	if entries.is_empty() {
		let card_id = tree.insert_child(
			list_id,
			Card::builder()
				.paddings(16.0)
				.corner_radius(12.0)
				.color(theme.monet[MonetRole::SurfaceContainerHigh])
				.build(),
		);
		tree.set_anchor(card_id, Some(Anchor::TopFill));

		let vbox_id = tree.insert_child(
			card_id,
			VBox::builder()
				.spacing(6.0)
				.color(Color::TRANSPARENT)
				.build(),
		);

		tree.insert_child(
			vbox_id,
			Text::builder()
				.text("Henüz sahne bulunamadı")
				.font_size(16.0)
				.color(theme.monet[MonetRole::Primary])
				.build(),
		);
		tree.insert_child(
			vbox_id,
			Text::builder()
				.text("Soldaki 'Yeni Proje' sekmesinden hemen bir proje oluşturabilirsiniz.")
				.font_size(13.0)
				.color(theme.monet[MonetRole::OnSurfaceVariant])
				.build(),
		);
		return cards;
	}

	for entry in entries {
		let card_id = tree.insert_child(
			list_id,
			Card::builder()
				.paddings(16.0)
				.corner_radius(12.0)
				.color(theme.monet[MonetRole::SurfaceContainerHigh])
				.build(),
		);
		tree.set_anchor(card_id, Some(Anchor::TopFill));

		let vbox_id = tree.insert_child(
			card_id,
			VBox::builder()
				.spacing(6.0)
				.color(Color::TRANSPARENT)
				.build(),
		);
		tree.set_anchor(vbox_id, Some(Anchor::TopFill));

		tree.insert_child(
			vbox_id,
			Text::builder()
				.text(entry.title.clone())
				.font_size(16.0)
				.color(theme.monet[MonetRole::Primary])
				.build(),
		);

		tree.insert_child(
			vbox_id,
			Text::builder()
				.text(entry.display_path.clone())
				.font_size(13.0)
				.color(theme.monet[MonetRole::OnSurfaceVariant])
				.build(),
		);

		let actions_row = tree.insert_child(
			vbox_id,
			HBox::builder()
				.spacing(8.0)
				.color(Color::TRANSPARENT)
				.build(),
		);
		tree.set_anchor(actions_row, Some(Anchor::TopFill));

		let spacer_id = tree.insert_child(actions_row, Spacer::default());
		tree.set_anchor(spacer_id, Some(Anchor::Fill));

		// Run button
		let run_entry = entry.clone();
		let run_state = Arc::clone(state);
		tree.insert_child(
			actions_row,
			Button::builder()
				.text("Çalıştır")
				.variant(ButtonVariant::Outlined)
				.on_click(move || {
					let opts = run_state
						.lock()
						.unwrap()
						.launch_options
						.lock()
						.unwrap()
						.clone();
					projects::launch_scene(&run_entry, &opts);
				})
				.build(),
		);

		// Open in Editor button
		let edit_entry = entry.clone();
		let edit_state = Arc::clone(state);
		let edit_tasks = tasks.clone();
		tree.insert_child(
			actions_row,
			Button::builder()
				.text("Editör")
				.variant(ButtonVariant::Tonal)
				.on_click(move || {
					let (launch_options, vp_ctrl, theme) = {
						let guard = edit_state.lock().unwrap();
						(
							Arc::clone(&guard.launch_options),
							guard.vp_ctrl.clone(),
							guard.theme,
						)
					};
					let entry_for_editor = edit_entry.clone();
					let back_state = Arc::clone(&edit_state);
					let back_tasks = edit_tasks.clone();
					let tasks_for_editor = edit_tasks.clone();
					edit_tasks.push_tree(move |tree| {
						editor::open_editor(
							tree,
							entry_for_editor,
							launch_options,
							vp_ctrl,
							theme,
							tasks_for_editor,
							Arc::new(move |tree| {
								let _ = mount_welcome(tree, &back_state, &back_tasks);
							}),
						);
					});
				})
				.build(),
		);

		cards.push((card_id, entry.clone()));
	}

	cards
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_welcome_qdl_loads_and_switches_tabs() {
		let mut registry = WidgetRegistry::new();
		nimbus::ui::register_widgets(&mut registry);

		let monet = Monet::new(
			Color::from_oklch(0.675, 0.138, 65.0),
			Color::from_oklch(0.563, 0.038, 65.0),
			Color::from_oklch(0.517, 0.052, 300.0),
			true,
		);
		let theme = Theme {
			elevation: ElevationTokens::default(),
			radius: RadiusTokens::default(),
			state: StateTokens::default(),
			stroke: StrokeTokens::default(),
			monet,
		};

		let mut tree = UiTree::new();
		let tasks = tree.task_queue();
		let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		let workspace_root = manifest_dir.parent().unwrap().parent().unwrap().to_path_buf();
		let state = Arc::new(Mutex::new(StudioState::new(
			workspace_root,
			theme,
			ViewportController::new_dummy(),
		)));
		let methods = build_method_table(Arc::clone(&state), tasks.clone());

		let scene = nimbus::qdl::load_into_tree_with_registry(
			include_str!("welcome.qdl"),
			&mut tree,
			&theme,
			&methods,
			&registry,
		)
		.expect("welcome.qdl should parse and load");

		let ids = extract_welcome_ids(&scene).expect("all named nodes should exist");
		state.lock().unwrap().ids = Some(ids);

		reload_scene_lists(&mut tree, &state, &tasks);
		assert!(!tree.nodes.get(ids.scene_list).unwrap().children.is_empty());

		// Switch tabs via switch_tab
		switch_tab(&mut tree, ids, WelcomeTab::Settings);
		assert!(!tree.is_visible(ids.page_projects));
		assert!(tree.is_visible(ids.page_settings));
		assert_eq!(
			tree.get_widget::<Button>(ids.nav_settings).unwrap().variant,
			ButtonVariant::Tonal
		);
		assert_eq!(
			tree.get_widget::<Button>(ids.nav_projects).unwrap().variant,
			ButtonVariant::Outlined
		);
	}

	#[test]
	fn test_editor_opens_real_scene() {
		{
			let mut reg = WidgetRegistry::global().write().unwrap();
			nimbus::ui::register_widgets(&mut reg);
			nimbus_ui_gizmo::register_widgets(&mut reg);
		}

		let monet = Monet::new(
			Color::from_oklch(0.675, 0.138, 65.0),
			Color::from_oklch(0.563, 0.038, 65.0),
			Color::from_oklch(0.517, 0.052, 300.0),
			true,
		);
		let theme = Theme {
			elevation: ElevationTokens::default(),
			radius: RadiusTokens::default(),
			state: StateTokens::default(),
			stroke: StrokeTokens::default(),
			monet,
		};

		let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
		let discovered = projects::discover_scenes(workspace_root);
		let entry = discovered
			.into_iter()
			.find(|s| s.display_path.ends_with("physics_cubes.mscene.ron"))
			.expect("physics_cubes.mscene.ron should be discovered");

		let mut tree = UiTree::new();
		let tasks = tree.task_queue();
		let opts = Arc::new(Mutex::new(LaunchOptions::default()));
		let vp_ctrl = ViewportController::new_dummy();

		editor::open_editor(
			&mut tree,
			entry,
			opts,
			vp_ctrl,
			theme,
			tasks,
			Arc::new(|_| {}),
		);

		assert!(!tree.roots.is_empty(), "Editor root should be mounted");
	}
}
