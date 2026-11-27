/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;


#[derive(Clone, Debug)]
pub struct LaunchOptions {
	pub vsync: bool,
	pub validation: bool,
	pub mute: bool,
	pub frame_gen: bool,
}

impl Default for LaunchOptions {
	fn default() -> Self {
		Self {
			vsync: true,
			validation: false,
			mute: false,
			frame_gen: false,
		}
	}
}

#[derive(Clone, Debug)]
pub struct SceneEntry {
	pub title: String,
	pub project_dir: PathBuf,
	pub scene_path: PathBuf,
	pub display_path: String,
}


pub fn discover_scenes(workspace_root: &Path) -> Vec<SceneEntry> {
	let mut entries = Vec::new();

	// Scan current directory if it's a Mochi project
	if workspace_root.join("mochi.ron").is_file() {
		collect_project_scenes(workspace_root, workspace_root, &mut entries);
	}

	// Scan standard project folders
	for folder in ["projects", "test", "examples", "external_scenes"] {
		let dir = workspace_root.join(folder);
		if let Ok(read_dir) = fs::read_dir(&dir) {
			for item in read_dir.flatten() {
				let path = item.path();
				if path.is_dir() {
					collect_project_scenes(workspace_root, &path, &mut entries);
				}
			}
		}
	}

	entries.sort_by(|a, b| a.display_path.cmp(&b.display_path));
	entries
}


fn collect_project_scenes(workspace_root: &Path, project_dir: &Path, out: &mut Vec<SceneEntry>) {
	let project_name = read_project_name(&project_dir.join("mochi.ron")).unwrap_or_else(|| {
		project_dir
			.file_name()
			.and_then(|s| s.to_str())
			.map(humanize_name)
			.unwrap_or_else(|| "Mochi Projesi".to_string())
	});

	let mut scene_files = Vec::new();

	// Check scenes/ subdirectory
	let scenes_dir = project_dir.join("scenes");
	if let Ok(read_dir) = fs::read_dir(&scenes_dir) {
		for item in read_dir.flatten() {
			let path = item.path();
			if is_mscene(&path) {
				scene_files.push(path);
			}
		}
	}

	// Also check root of project_dir if no scenes found in scenes/
	if scene_files.is_empty()
		&& let Ok(read_dir) = fs::read_dir(project_dir)
	{
		for item in read_dir.flatten() {
			let path = item.path();
			if is_mscene(&path) {
				scene_files.push(path);
			}
		}
	}

	let multiple = scene_files.len() > 1;
	for scene_path in scene_files {
		let scene_stem = scene_path
			.file_name()
			.and_then(|s| s.to_str())
			.and_then(|s| s.strip_suffix(".mscene.ron"))
			.map(humanize_name)
			.unwrap_or_else(|| "Sahne".to_string());

		let title = if multiple {
			format!("{project_name} — {scene_stem}")
		} else {
			project_name.clone()
		};

		let display_path = scene_path
			.strip_prefix(workspace_root)
			.unwrap_or(&scene_path)
			.to_string_lossy()
			.replace('\\', "/");

		out.push(SceneEntry {
			title,
			project_dir: project_dir.to_path_buf(),
			scene_path,
			display_path,
		});
	}
}


fn is_mscene(path: &Path) -> bool {
	path.is_file()
		&& path
			.file_name()
			.and_then(|s| s.to_str())
			.is_some_and(|name| name.ends_with(".mscene.ron"))
}


fn read_project_name(mochi_ron: &Path) -> Option<String> {
	let content = fs::read_to_string(mochi_ron).ok()?;
	for line in content.lines() {
		let trimmed = line.trim();
		if let Some(rest) = trimmed.strip_prefix("name:") {
			let val = rest.trim().trim_end_matches(',').trim();
			if let Some(unquoted) = val.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
				return Some(unquoted.to_string());
			}
		}
	}
	None
}


fn humanize_name(raw: &str) -> String {
	raw
		.split(['_', '-'])
		.filter(|part| !part.is_empty())
		.map(|part| {
			let mut chars = part.chars();
			match chars.next() {
				Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
				None => String::new(),
			}
		})
		.collect::<Vec<_>>()
		.join(" ")
}


pub fn launch_scene(entry: &SceneEntry, options: &LaunchOptions) {
	let exe = match std::env::current_exe() {
		Ok(path) => path,
		Err(e) => {
			eprintln!("[Mochi Studio] Çalıştırılabilir dosya bulunamadı: {e}");
			return;
		}
	};

	println!(
		"[Mochi Studio] Başlatılıyor: {} ({})",
		entry.title, entry.display_path
	);

	let mut cmd = Command::new(exe);
	cmd
		.arg("run")
		.arg(&entry.project_dir)
		.arg("--scene")
		.arg(&entry.scene_path);

	if options.vsync {
		cmd.arg("--vsync");
	} else {
		cmd.arg("--no-vsync");
	}
	if options.validation {
		cmd.arg("--validation");
	}
	if options.mute {
		cmd.arg("--mute");
	}
	if options.frame_gen {
		cmd.arg("--fg");
	}

	if let Err(e) = cmd.spawn() {
		eprintln!("[Mochi Studio] Sahne başlatılamadı: {e}");
	}
}


pub fn launch_editor(scene_path: &Path) {
	let exe = match std::env::current_exe() {
		Ok(path) => path,
		Err(e) => {
			eprintln!("[Mochi Studio] Çalıştırılabilir dosya bulunamadı: {e}");
			return;
		}
	};

	let mut cmd = Command::new(exe);
	cmd.arg("studio").arg("--scene").arg(scene_path);

	if let Err(e) = cmd.spawn() {
		eprintln!("[Mochi Studio] Editör penceresi başlatılamadı: {e}");
	}
}


pub fn launch_hub() {
	let exe = match std::env::current_exe() {
		Ok(path) => path,
		Err(e) => {
			eprintln!("[Mochi Studio] Çalıştırılabilir dosya bulunamadı: {e}");
			return;
		}
	};

	let mut cmd = Command::new(exe);
	cmd.arg("studio");

	if let Err(e) = cmd.spawn() {
		eprintln!("[Mochi Studio] Hub penceresi başlatılamadı: {e}");
	}
}


pub fn create_project(
	target_path: &str,
	name: &str,
	desc: &str,
	no_git: bool,
) -> Result<String, String> {
	let trimmed_path = target_path.trim();
	if trimmed_path.is_empty() {
		return Err("Hedef dizin boş olamaz.".to_string());
	}

	let exe =
		std::env::current_exe().map_err(|e| format!("Çalıştırılabilir dosya bulunamadı: {e}"))?;
	let mut cmd = Command::new(exe);
	cmd.arg("init").arg(trimmed_path);

	if !name.trim().is_empty() {
		cmd.arg("--name").arg(name.trim());
	}
	if !desc.trim().is_empty() {
		cmd.arg("--desc").arg(desc.trim());
	}
	if no_git {
		cmd.arg("--no-git");
	}

	let output = cmd
		.output()
		.map_err(|e| format!("'mochi init' çalıştırılamadı: {e}"))?;

	if output.status.success() {
		Ok(format!("Proje oluşturuldu: {trimmed_path}"))
	} else {
		let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
		if stderr.is_empty() {
			Err(format!(
				"Proje oluşturulamadı (kod: {:?})",
				output.status.code()
			))
		} else {
			Err(stderr)
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_discover_workspace_scenes() {
		let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
		let scenes = discover_scenes(workspace_root);
		assert!(
			scenes
				.iter()
				.any(|s| s.display_path.ends_with("physics_cubes.mscene.ron")),
			"Expected to discover physics_cubes.mscene.ron, found: {scenes:?}"
		);
	}
}
