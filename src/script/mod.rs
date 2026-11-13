/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod api;
pub mod qw;

use std::collections::{HashMap, HashSet};
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::core::input::Input;
use crate::scene::Scene;
use hecs::Entity;
use mochi_except::*;
use qw::LoadedQwScript;
use api::ContextGuard;


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptComponent {
	pub path: String,
	pub is_active: bool,
}

impl ScriptComponent {
	pub fn new(path: impl Into<String>) -> Self {
		Self {
			path: path.into(),
			is_active: true,
		}
	}
}

pub struct ScriptSystem {
	loaded_scripts: HashMap<String, LoadedQwScript>,
	started_entities: HashSet<Entity>,
}

impl ScriptSystem {
	pub fn new() -> Self {
		Self {
			loaded_scripts: HashMap::new(),
			started_entities: HashSet::new(),
		}
	}

	pub fn update(&mut self, scene: &mut Scene, input: &Input, dt: f32, time: f32) {
		// Collect entities with scripts to avoid borrow conflicts
		let mut script_entities = Vec::new();
		for (entity, script) in scene.world().query::<&ScriptComponent>().iter() {
			if script.is_active {
				script_entities.push((entity, script.path.clone()));
			}
		}

		if script_entities.is_empty() {
			return;
		}

		// Activate C-ABI context guard so QW functions can access the world and input
		let _guard = ContextGuard::new(scene.world_mut(), input, time);

		for (entity, path_str) in script_entities {
			let path = Path::new(&path_str);
			if !path.exists() {
				continue;
			}

			let script = self
				.loaded_scripts
				.entry(path_str.clone())
				.or_insert_with(|| match LoadedQwScript::load(path) {
					Ok(loaded) => loaded,
					Err(err) => {
						mochi_error!("Failed to load script {}: {}", path.display(), err);
						LoadedQwScript::empty(path)
					}
				});

			if let Ok(true) = script.reload_if_changed() {
				mochi_disp!("Hot-reloaded script '{}'", path_str);
			}

			let entity_bits = entity.to_bits().get();
			if !self.started_entities.contains(&entity) {
				script.call_start(entity_bits);
				self.started_entities.insert(entity);
			}

			script.call_update(entity_bits, dt);
		}
	}
}

impl Default for ScriptSystem {
	fn default() -> Self {
		Self::new()
	}
}
