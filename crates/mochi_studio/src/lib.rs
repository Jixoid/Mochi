/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod projects;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use nimbus::core::{
	Anchor, Application, MethodTable, ObjectID, TaskQueue, UiTree, WidgetRegistry, Window,
};
use nimbus::math::Color;
use nimbus::theme::{
	ElevationTokens, Monet, MonetRole, RadiusTokens, StateTokens, StrokeTokens, Theme,
};
use nimbus::ui::{Button, ButtonVariant, Card, Text, VBox};

use crate::projects::{LaunchOptions, SceneEntry};


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
	launch_options: LaunchOptions,
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
	fn new(cwd: PathBuf, theme: Theme) -> Self {
		Self {
			ids: None,
			launch_options: LaunchOptions::default(),
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
	nimbus::ui::register_widgets(&mut WidgetRegistry::global().write().unwrap());

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

	let window_handle = app.create_window(Window::new("Mochi Studio", 980, 640));
	let main_window = app
		.window_mut(window_handle)
		.ok_or("Failed to get main window")?;

	let tasks = main_window.task_queue();
	let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
	let state = Arc::new(Mutex::new(StudioState::new(cwd, theme)));

	let methods = build_method_table(Arc::clone(&state), tasks);

	let qdl_scene = nimbus::qdl::load_into_tree(
		include_str!("welcome.qdl"),
		main_window.tree_mut(),
		&theme,
		&methods,
	)
	.map_err(|e| format!("Failed to parse welcome.qdl: {e}"))?;

	let ids = extract_welcome_ids(&qdl_scene)?;
	{
		let mut guard = state.lock().unwrap();
		guard.ids = Some(ids);
	}

	reload_scene_lists(main_window.tree_mut(), &state);

	if let Some(scene_path) = initial_scene {
		println!("[Mochi Studio] Doğrudan sahne açılması istendi: {scene_path:?}");
	}

	app.run().map_err(|e| format!("Application error: {e}"))?;
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
			tasks.push_tree(move |tree| {
				reload_scene_lists(tree, &state);
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

			if let Some(ids) = ids {
				tasks.push_tree(move |tree| {
					let (msg, color, ok) = match result {
						Ok(ok_msg) => (ok_msg, theme.monet[MonetRole::Primary], true),
						Err(err_msg) => (err_msg, theme.monet[MonetRole::Error], false),
					};

					if let Some(status_text) = tree.get_widget_mut::<Text>(ids.create_status) {
						status_text.text = msg;
						status_text.color = Some(color);
						tree.needs_layout = true;
						tree.needs_redraw = true;
					}

					if ok {
						reload_scene_lists(tree, &state_clone);
					}
				});
			}
		});
	}

	// Settings switches
	{
		let state = Arc::clone(&state);
		methods.register("OnVsyncChanged", move |checked: bool| {
			state.lock().unwrap().launch_options.vsync = checked;
		});
	}
	{
		let state = Arc::clone(&state);
		methods.register("OnValidationChanged", move |checked: bool| {
			state.lock().unwrap().launch_options.validation = checked;
		});
	}
	{
		let state = Arc::clone(&state);
		methods.register("OnMuteChanged", move |checked: bool| {
			state.lock().unwrap().launch_options.mute = checked;
		});
	}
	{
		let state = Arc::clone(&state);
		methods.register("OnFgChanged", move |checked: bool| {
			state.lock().unwrap().launch_options.frame_gen = checked;
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


fn reload_scene_lists(tree: &mut UiTree, state: &Arc<Mutex<StudioState>>) {
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

	// Rebuild main Projects/All Scenes list
	clear_children(tree, ids.scene_list);
	let created_cards = populate_scene_cards(tree, ids.scene_list, &discovered, &theme, state);

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

		let entry_clone = entry.clone();
		let state_clone = Arc::clone(state);
		let btn_id = tree.insert_child(
			vbox_id,
			Button::builder()
				.text("Aç")
				.variant(ButtonVariant::Tonal)
				.on_click(move || {
					let opts = state_clone.lock().unwrap().launch_options.clone();
					projects::launch_scene(&entry_clone, &opts);
				})
				.build(),
		);
		tree.set_anchor(btn_id, Some(Anchor::RightFill));

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
		let state = Arc::new(Mutex::new(StudioState::new(workspace_root, theme)));
		let methods = build_method_table(Arc::clone(&state), tasks);

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

		reload_scene_lists(&mut tree, &state);
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
}
