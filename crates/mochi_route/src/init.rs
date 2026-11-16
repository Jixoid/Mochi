/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{env, fs, path::PathBuf};
use owo_colors::OwoColorize;

use crate::Error;


pub struct InitInfo {
	pub path: String,
	pub name: String,
	pub desc: String,
	pub no_git: bool,
	pub force: bool,
}

pub fn init(info: InitInfo) -> Result<(), Error> {
	let path = if info.path.is_empty() {
		env::current_dir()?
	} else {
		PathBuf::from(&info.path)
	};

	if !path.exists() {
		fs::create_dir_all(&path)?;
	}
	if !path.is_dir() {
		return Err(Error::New | format!("the path is not a directory: {}", path.display()));
	}

	let is_not_empty = fs::read_dir(&path)?.next().is_some();
	if is_not_empty && !info.force {
		return Err(Error::New | "the directory is not empty");
	}

	let proj_name = if info.name.is_empty() {
		path
			.canonicalize()
			.ok()
			.and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
			.unwrap_or_else(|| "MyGame".to_string())
	} else {
		info.name
	};

	// 1. Write mochi.ron
	let mochi_conf = format!(
		r#"ProjectConfig(
	name: "{proj_name}",
	version: "0.1.0",
	window: WindowConfig(
		title: "{proj_name}",
		width: 1280,
		height: 720,
		vsync: true,
	),
	rendering: RenderingConfig(
		clear_color: (0.1, 0.15, 0.25, 1.0),
	),
	scene: SceneConfig(
		start_scene: "scenes/main.mscene.ron",
	),
)
"#
	);
	fs::write(path.join("mochi.ron"), mochi_conf)?;

	// 2. Create scenes/ directory with minimal template scene
	let scenes_dir = path.join("scenes");
	fs::create_dir_all(&scenes_dir)?;

	let main_scene = r#"SceneAsset(
	name: "MainScene",
	entities: [
		EntityDesc(
			name: Some("MainCamera"),
			transform: Some(TransformDesc(
				translation: (0.0, 1.5, 4.0),
			)),
			camera: Some(CameraDesc(
				fov_deg: 60.0,
				near: 0.1,
				far: 1000.0,
				is_active: true,
				clear_color: (0.04, 0.05, 0.08, 1.0),
			)),
			script: Some(ScriptDesc(
				path: "scripts/main.qw",
				is_active: true,
			)),
		),
		EntityDesc(
			name: Some("DirectionalLight"),
			transform: Some(TransformDesc(
				translation: (0.0, 10.0, 0.0),
			)),
			directional_light: Some(DirectionalLightDesc(
				color: (1.0, 0.95, 0.9),
				intensity: 1.0,
				direction: (0.5, -1.0, 0.3),
				cast_shadows: true,
			)),
		),
	],
)
"#;
	fs::write(scenes_dir.join("main.mscene.ron"), main_scene)?;

	// 3. Create scripts/ directory with main.qw
	let scripts_dir = path.join("scripts");
	fs::create_dir_all(&scripts_dir)?;

	let main_qw = r#"use mochi::*;

![C, export]
pub fun on_start(entity: u64) {
	// Called once when entity is initialized
}

![C, export]
pub fun on_update(entity: u64, dt: f32) {
	// Called every frame
}
"#;
	fs::write(scripts_dir.join("main.qw"), main_qw)?;

	// 4. Git init
	if !info.no_git {
		let _ = fs::write(path.join(".gitignore"), "/target\n*.so\n");
		let status = std::process::Command::new("git")
			.arg("-C")
			.arg(&path)
			.args(["init", "-q"])
			.status()
			.map_err(|e| Error::New | format!("git could not be initialized: {e}"))?;

		if !status.success() {
			return Err(Error::New | "git repo cannot be initialized");
		}
	}

	eprintln!(
		"{:>12} project '{}' at {}",
		"Created".green().bold(),
		proj_name,
		path.display()
	);

	Ok(())
}
