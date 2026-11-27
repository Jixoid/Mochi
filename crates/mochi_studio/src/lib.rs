/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod editor;
pub mod hub;
pub mod projects;
pub mod scene_overlays;
pub mod viewport_renderer;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use nimbus::core::{Application, WidgetRegistry, Window};
use nimbus::math::Color;
use nimbus::theme::{ElevationTokens, Monet, RadiusTokens, StrokeTokens, Theme};

use crate::projects::{LaunchOptions, SceneEntry};
use crate::viewport_renderer::StudioViewportBridge;


pub fn run(initial_scene: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
	nimbus::except::start();
	{
		let mut reg = WidgetRegistry::global().write().unwrap();
		nimbus::ui::register_widgets(&mut reg);
		nimbus::ext::dock::register_widgets(&mut reg);
		nimbus::ext::gizmo::register_widgets(&mut reg);
		nimbus::ext::inspector::register_widgets(&mut reg);
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
		radius: RadiusTokens {
			none: 0.0,
			xsmall: 2.0,
			small: 4.0,
			medium: 6.0,
			large: 8.0,
			xlarge: 12.0,
			full: f32::MAX,
		},
		stroke: StrokeTokens::default(),
		monet,
	};

	match initial_scene {
		Some(scene_path) => run_editor_window(scene_path, theme),
		None => run_welcome_window(theme),
	}
}


fn run_welcome_window(theme: Theme) -> Result<(), Box<dyn std::error::Error>> {
	let mut app = Application::new(theme);
	let theme = app.theme;

	let window_handle = app.create_window(Window::new("Mochi Studio — Hub", 960, 620));
	let main_window = app
		.window_mut(window_handle)
		.ok_or("Failed to get welcome window")?;

	let tasks = main_window.task_queue();
	let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
	hub::mount_welcome(main_window.tree_mut(), cwd, theme, &tasks)?;

	app.run().map_err(|e| format!("Application error: {e}"))?;
	Ok(())
}


fn run_editor_window(scene_path: PathBuf, theme: Theme) -> Result<(), Box<dyn std::error::Error>> {
	let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
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

	let window_title = format!("Mochi Studio — {title}");
	let display_path = scene_path.to_string_lossy().replace('\\', "/");
	let entry = SceneEntry {
		title,
		project_dir,
		scene_path,
		display_path,
	};

	let mut app = Application::new(theme);
	let theme = app.theme;
	let app_tasks = app.task_queue();

	let window_handle = app.create_window(Window::new(window_title, 1360, 820));
	let main_window = app
		.window_mut(window_handle)
		.ok_or("Failed to get editor window")?;

	let tasks = main_window.task_queue();
	let (mut vp_bridge, vp_ctrl) = StudioViewportBridge::spawn(tasks.clone());
	let launch_options = Arc::new(Mutex::new(LaunchOptions::default()));

	editor::open_editor(
		main_window.tree_mut(),
		entry,
		launch_options,
		vp_ctrl,
		theme,
		tasks,
		app_tasks,
		Arc::new(projects::launch_hub),
	);

	app.set_on_pre_render(move |device, cmd_manager, window| {
		vp_bridge.poll(device, cmd_manager, window);
	});

	app.run().map_err(|e| format!("Application error: {e}"))?;
	Ok(())
}


#[cfg(test)]
mod tests {
	use super::*;
	use crate::viewport_renderer::ViewportController;
	use nimbus::core::{TaskQueue, UiTree};

	#[test]
	fn test_editor_opens_real_scene() {
		{
			let mut reg = WidgetRegistry::global().write().unwrap();
			nimbus::ui::register_widgets(&mut reg);
			nimbus::ext::dock::register_widgets(&mut reg);
			nimbus::ext::gizmo::register_widgets(&mut reg);
			nimbus::ext::inspector::register_widgets(&mut reg);
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
		let app_tasks = TaskQueue::new();
		let opts = Arc::new(Mutex::new(LaunchOptions::default()));
		let vp_ctrl = ViewportController::new_dummy();

		editor::open_editor(
			&mut tree,
			entry,
			opts,
			vp_ctrl,
			theme,
			tasks,
			app_tasks,
			Arc::new(|| {}),
		);

		assert!(!tree.roots.is_empty(), "Editor root should be mounted");
	}
}
