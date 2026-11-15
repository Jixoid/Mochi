/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::fs;
use std::path::Path;
use owo_colors::OwoColorize;

use crate::Error;


pub struct CleanInfo<'a> {
	pub path: &'a Path,
}

pub fn clean(info: CleanInfo) -> Result<(), Error> {
	let project_dir = match mochi_core::ProjectConfig::find_project_root(info.path) {
		Some(dir) => dir,
		None => {
			return Err(Error::New | format!("cannot find mochi.ron in '{}'", info.path.display()));
		}
	};

	let cache_dir = project_dir.join("target/qw_cache");
	if cache_dir.exists() {
		fs::remove_dir_all(&cache_dir)?;
		eprintln!("{:>12} {}", "Cleaned".green().bold(), cache_dir.display());
	}
	let build_dir = project_dir.join("build");
	if build_dir.exists() {
		fs::remove_dir_all(&build_dir)?;
		eprintln!("{:>12} {}", "Cleaned".green().bold(), build_dir.display());
	}
	Ok(())
}
