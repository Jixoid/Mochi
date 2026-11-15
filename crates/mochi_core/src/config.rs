/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

fn default_root_dir() -> PathBuf {
	PathBuf::from(".")
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WindowConfig {
	pub title: String,
	pub width: u32,
	pub height: u32,
	pub vsync: bool,
}

impl Default for WindowConfig {
	fn default() -> Self {
		Self {
			title: "Mochi Engine".to_string(),
			width: 1280,
			height: 720,
			vsync: true,
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderingConfig {
	pub clear_color: [f32; 4],
}

impl Default for RenderingConfig {
	fn default() -> Self {
		Self {
			clear_color: [0.1, 0.15, 0.25, 1.0],
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneConfig {
	pub start_scene: String,
}

impl Default for SceneConfig {
	fn default() -> Self {
		Self {
			start_scene: "scenes/main.mscene.ron".to_string(),
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
	pub name: String,
	pub version: String,
	pub window: WindowConfig,
	pub rendering: RenderingConfig,
	pub scene: SceneConfig,
	#[serde(skip, default = "default_root_dir")]
	pub root_dir: PathBuf,
}

impl Default for ProjectConfig {
	fn default() -> Self {
		Self {
			name: "Mochi Engine".to_string(),
			version: "0.1.0".to_string(),
			window: WindowConfig::default(),
			rendering: RenderingConfig::default(),
			scene: SceneConfig::default(),
			root_dir: default_root_dir(),
		}
	}
}

impl ProjectConfig {
	pub fn find_project_root(start: impl AsRef<Path>) -> Option<PathBuf> {
		let p = start.as_ref();
		let mut current = if p.is_file() {
			p.parent()
		} else {
			Some(p)
		};
		while let Some(dir) = current {
			if dir.join("mochi.ron").exists() {
				return Some(dir.to_path_buf());
			}
			current = dir.parent();
		}
		None
	}

	pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
		let p = path.as_ref();
		let (target_file, root_dir) = if p.is_file() {
			let root = Self::find_project_root(p)
				.ok_or_else(|| format!("cannot find mochi.ron in '{}'", p.display()))?;
			(p.to_path_buf(), root)
		} else {
			let root = Self::find_project_root(p)
				.ok_or_else(|| format!("cannot find mochi.ron in '{}'", p.display()))?;
			(root.join("mochi.ron"), root)
		};

		if !target_file.exists() {
			return Err(format!("cannot find mochi.ron in '{}'", root_dir.display()));
		}

		let content = fs::read_to_string(&target_file)
			.map_err(|e| format!("cannot read '{}': {e}", target_file.display()))?;

		let mut config = ron::from_str::<Self>(&content)
			.map_err(|e| format!("cannot parse '{}': {e}", target_file.display()))?;

		config.root_dir = root_dir;
		Ok(config)
	}

	pub fn load_or_default(path: &str) -> Self {
		match Self::load(path) {
			Ok(cfg) => cfg,
			Err(err) => {
				eprintln!("error: {err}");
				std::process::exit(1);
			}
		}
	}

	pub fn resolve_path(&self, rel_path: impl AsRef<Path>) -> PathBuf {
		let p = rel_path.as_ref();
		if p.is_absolute() {
			p.to_path_buf()
		} else {
			self.root_dir.join(p)
		}
	}

	pub fn save_to_file(&self, path: &str) -> Result<(), String> {
		let ron_str = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
			.map_err(|e| format!("Failed to serialize config: {e}"))?;
		fs::write(path, ron_str).map_err(|e| format!("Failed to write config file: {e}"))
	}
}
