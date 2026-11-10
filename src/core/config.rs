/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};


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
			start_scene: "assets/scenes/main.mscene.ron".to_string(),
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
}

impl Default for ProjectConfig {
	fn default() -> Self {
		Self {
			name: "Mochi Engine".to_string(),
			version: "0.1.0".to_string(),
			window: WindowConfig::default(),
			rendering: RenderingConfig::default(),
			scene: SceneConfig::default(),
		}
	}
}

impl ProjectConfig {
	pub fn load_or_default(path: &str) -> Self {
		if Path::new(path).exists() {
			match fs::read_to_string(path) {
				Ok(content) => match ron::from_str::<Self>(&content) {
					Ok(config) => return config,
					Err(err) => eprintln!("Failed to parse config {path}: {err}. Using default config."),
				},
				Err(err) => eprintln!("Failed to read config {path}: {err}. Using default config."),
			}
		}
		Self::default()
	}

	pub fn save_to_file(&self, path: &str) -> Result<(), String> {
		let ron_str = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
			.map_err(|e| format!("Failed to serialize config: {e}"))?;
		fs::write(path, ron_str).map_err(|e| format!("Failed to write config file: {e}"))
	}
}
