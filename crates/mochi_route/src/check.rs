/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::fs;
use std::path::Path;
use mochi_asset::SceneAsset;
use mochi_core::ProjectConfig;
use owo_colors::OwoColorize;

use crate::Error;


pub struct CheckInfo<'a> {
	pub path: &'a Path,
}

pub fn check(info: CheckInfo) -> Result<(), Error> {
	eprintln!("{:>12} {}", "Checking".green().bold(), info.path.display());

	let project_dir = match ProjectConfig::find_project_root(info.path) {
		Some(dir) => dir,
		None => {
			return Err(Error::New | format!("cannot find mochi.ron in '{}'", info.path.display()));
		}
	};

	let config_path = project_dir.join("mochi.ron");
	if !config_path.exists() {
		return Err(Error::New | format!("cannot find mochi.ron in {}", project_dir.display()));
	}

	let content = fs::read_to_string(&config_path)
		.map_err(|e| Error::New | format!("cannot read {}: {e}", config_path.display()))?;

	let cfg = ron::from_str::<ProjectConfig>(&content)
		.map_err(|e| Error::New | format!("cannot parse {}: {e}", config_path.display()))?;

	// Check start scene
	let scene_path = mochi_asset::resolve_relative(&cfg.scene.start_scene, None, Some(&project_dir));
	if !scene_path.exists() {
		return Err(Error::New | format!("start scene does not exist: {}", scene_path.display()));
	}

	// Check assets inside start scene
	if let Ok(scene_asset) = SceneAsset::load_file(&scene_path) {
		let scene_dir = scene_path.parent();
		for entity in &scene_asset.entities {
			if let Some(ref mr) = entity.mesh_renderer {
				let model_path = mochi_asset::resolve_relative(&mr.asset, scene_dir, Some(&project_dir));
				if !model_path.exists() {
					eprintln!(
						"{}{} missing model referenced in scene: {}",
						"warning".yellow().bold(),
						":".bright_black(),
						model_path.display()
					);
				}
			}
			if let Some(ref sc) = entity.script {
				let script_path = mochi_asset::resolve_relative(&sc.path, scene_dir, Some(&project_dir));
				if !script_path.exists() {
					eprintln!(
						"{}{} missing script referenced in scene: {}",
						"warning".yellow().bold(),
						":".bright_black(),
						script_path.display()
					);
				}
			}
		}
	}

	// Check QW compiler
	match std::process::Command::new("qw.debug")
		.arg("--version")
		.output()
	{
		Ok(out) if out.status.success() => (),
		_ => {
			eprintln!(
				"{}{} 'qw.debug' compiler binary not found in PATH",
				"warning".yellow().bold(),
				":".bright_black()
			);
		}
	}

	eprintln!(
		"{:>12} project '{}' v{}",
		"Finished".green().bold(),
		cfg.name,
		cfg.version
	);

	Ok(())
}
