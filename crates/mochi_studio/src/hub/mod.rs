/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use nimbus::core::{Anchor, MethodTable, ObjectID, TaskQueue, UiTree};
use nimbus::math::Color;
use nimbus::theme::{DColor, MonetRole, Theme};
use nimbus::ui::{Button, ButtonVariant, Card, HBox, Spacer, Text, Toast, ToastKind, VBox};

use crate::projects::{self, LaunchOptions, SceneEntry};


#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum WelcomeTab {
	Projects,
	NewProject,
	Settings,
}

#[derive(Clone, Copy)]
pub(crate) struct WelcomeIds {
	pub nav_projects: ObjectID,
	pub nav_new_scene: ObjectID,
	pub nav_settings: ObjectID,
	pub page_projects: ObjectID,
	pub page_new_scene: ObjectID,
	pub page_settings: ObjectID,
	pub scene_list: ObjectID,
	pub create_status: ObjectID,
}

pub(crate) struct HubState {
	pub ids: Option<WelcomeIds>,
	pub launch_options: Arc<Mutex<LaunchOptions>>,
	pub project_name: String,
	pub project_path: String,
	pub project_desc: String,
	pub project_no_git: bool,
	pub search_query: String,
	pub scene_cards: Vec<(ObjectID, SceneEntry)>,
	pub cwd: PathBuf,
}

impl HubState {
	pub fn new(cwd: PathBuf) -> Self {
		Self {
			ids: None,
			launch_options: Arc::new(Mutex::new(LaunchOptions::default())),
			project_name: "my_mochi_game".to_string(),
			project_path: "./projects/my_mochi_game".to_string(),
			project_desc: "Mochi 3D Engine projesi".to_string(),
			project_no_git: false,
			search_query: String::new(),
			scene_cards: Vec::new(),
			cwd,
		}
	}
}


pub fn mount_welcome(
	tree: &mut UiTree,
	cwd: PathBuf,
	theme: Theme,
	tasks: &TaskQueue,
) -> Result<(), String> {
	let state = Arc::new(Mutex::new(HubState::new(cwd)));
	let methods = build_method_table(Arc::clone(&state), tasks.clone());

	let qdl_scene = nimbus::qdl::load_into_tree(
		include_str!("../../design/welcome.qdl"),
		tree,
		&theme,
		&methods,
	)
	.map_err(|e| format!("Failed to parse welcome.qdl: {e}"))?;

	let ids = extract_welcome_ids(&qdl_scene)?;
	state.lock().unwrap().ids = Some(ids);

	reload_scene_lists(tree, &state, tasks);
	Ok(())
}


pub(crate) fn extract_welcome_ids(scene: &nimbus::qdl::QdlScene) -> Result<WelcomeIds, String> {
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


pub(crate) fn build_method_table(state: Arc<Mutex<HubState>>, tasks: TaskQueue) -> MethodTable {
	let mut methods = MethodTable::new();

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
			let (path, name, desc, no_git, ids) = {
				let guard = state.lock().unwrap();
				(
					guard.project_path.clone(),
					guard.project_name.clone(),
					guard.project_desc.clone(),
					guard.project_no_git,
					guard.ids,
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
							DColor::from(MonetRole::Primary),
							true,
						),
						Err(err_msg) => (
							"Proje Hatası",
							ToastKind::Error,
							err_msg,
							DColor::from(MonetRole::Error),
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

	{
		let state = Arc::clone(&state);
		methods.register("OnVsyncChanged", move |checked: bool| {
			state.lock().unwrap().launch_options.lock().unwrap().vsync = checked;
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
			state.lock().unwrap().launch_options.lock().unwrap().mute = checked;
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


pub(crate) fn switch_tab(tree: &mut UiTree, ids: WelcomeIds, active: WelcomeTab) {
	let tabs = [
		(WelcomeTab::Projects, ids.nav_projects, ids.page_projects),
		(
			WelcomeTab::NewProject,
			ids.nav_new_scene,
			ids.page_new_scene,
		),
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


pub(crate) fn reload_scene_lists(
	tree: &mut UiTree,
	state: &Arc<Mutex<HubState>>,
	tasks: &TaskQueue,
) {
	let (ids, cwd, needle) = {
		let guard = state.lock().unwrap();
		match guard.ids {
			Some(ids) => (
				ids,
				guard.cwd.clone(),
				guard.search_query.trim().to_lowercase(),
			),
			None => return,
		}
	};

	let discovered = projects::discover_scenes(&cwd);

	clear_children(tree, ids.scene_list);
	let created_cards = populate_scene_cards(tree, ids.scene_list, &discovered, state, tasks);

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
	state: &Arc<Mutex<HubState>>,
	tasks: &TaskQueue,
) -> Vec<(ObjectID, SceneEntry)> {
	let mut cards = Vec::with_capacity(entries.len());

	if entries.is_empty() {
		let card_id = tree.insert_child(
			list_id,
			Card::builder()
				.paddings(16.0)
				.corner_radius(12.0)
				.color(DColor::from(MonetRole::SurfaceContainerHigh))
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
				.color(DColor::from(MonetRole::Primary))
				.build(),
		);
		tree.insert_child(
			vbox_id,
			Text::builder()
				.text("Soldaki 'Yeni Proje' sekmesinden hemen bir proje oluşturabilirsiniz.")
				.font_size(13.0)
				.color(DColor::from(MonetRole::OnSurfaceVariant))
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
				.color(DColor::from(MonetRole::SurfaceContainerHigh))
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
				.color(DColor::from(MonetRole::Primary))
				.build(),
		);

		tree.insert_child(
			vbox_id,
			Text::builder()
				.text(entry.display_path.clone())
				.font_size(13.0)
				.color(DColor::from(MonetRole::OnSurfaceVariant))
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

		let edit_scene_path = entry.scene_path.clone();
		let edit_tasks = tasks.clone();
		tree.insert_child(
			actions_row,
			Button::builder()
				.text("Editör")
				.variant(ButtonVariant::Tonal)
				.on_click(move || {
					projects::launch_editor(&edit_scene_path);
					edit_tasks.push_window(|win| {
						win.close();
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
	use nimbus::core::WidgetRegistry;
	use nimbus::theme::{ElevationTokens, Monet, RadiusTokens, StrokeTokens};

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
			stroke: StrokeTokens::default(),
			monet,
		};

		let mut tree = UiTree::new();
		let tasks = tree.task_queue();
		let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		let workspace_root = manifest_dir
			.parent()
			.unwrap()
			.parent()
			.unwrap()
			.to_path_buf();
		let state = Arc::new(Mutex::new(HubState::new(workspace_root)));
		let methods = build_method_table(Arc::clone(&state), tasks.clone());

		let scene = nimbus::qdl::load_into_tree_with_registry(
			include_str!("../../design/welcome.qdl"),
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
