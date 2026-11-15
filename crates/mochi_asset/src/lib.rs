/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::path::{Path, PathBuf};

pub mod model;
pub mod scene;

pub use model::*;
pub use scene::*;


/// Resolves a path relative to an optional referencing file's directory and project root.
pub fn resolve_relative(
	rel: impl AsRef<Path>,
	file_dir: Option<&Path>,
	project_root: Option<&Path>,
) -> PathBuf {
	let p = rel.as_ref();
	if p.is_absolute() {
		return p.to_path_buf();
	}
	// 1. Relative to referencing file directory
	if let Some(dir) = file_dir {
		let candidate = dir.join(p);
		if candidate.exists() {
			return candidate;
		}
	}
	// 2. Relative to project root
	if let Some(root) = project_root {
		let candidate = root.join(p);
		if candidate.exists() {
			return candidate;
		}
	}
	// Fallback
	if let Some(root) = project_root {
		root.join(p)
	} else if let Some(dir) = file_dir {
		dir.join(p)
	} else {
		p.to_path_buf()
	}
}
