/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use owo_colors::OwoColorize;

use crate::Error;


pub struct BuildInfo<'a> {
	pub path: &'a Path,
	pub force: bool,
	pub verbose: u8,
}

pub fn build(info: BuildInfo) -> Result<(), Error> {
	let start_time = Instant::now();

	// 1. Ensure QW engine unit
	let _mochi_unit = mochi_script::qw::ensure_engine_unit()
		.map_err(|e| Error::New | format!("cannot prepare engine unit: {e}"))?;

	let mut scripts = Vec::new();
	collect_qw_files(info.path, &mut scripts);

	scripts.sort();
	scripts.dedup();

	if scripts.is_empty() {
		eprintln!(
			"{}{} no .qw scripts found in {}",
			"note".cyan().bold(),
			":".bright_black(),
			info.path.display()
		);
		return Ok(());
	}

	let project_dir = match mochi_core::ProjectConfig::find_project_root(info.path) {
		Some(dir) => dir,
		None => {
			return Err(Error::New | format!("cannot find mochi.ron in '{}'", info.path.display()));
		}
	};

	let cache_dir = project_dir.join("target/qw_cache");
	fs::create_dir_all(&cache_dir)?;

	let mut compiled_count = 0;

	for script in &scripts {
		let stem = script
			.file_stem()
			.and_then(|s| s.to_str())
			.unwrap_or("script");
		let out_so = cache_dir.join(format!("{stem}.so"));

		let needs_compile = info.force || !out_so.exists() || {
			if let (Ok(src_meta), Ok(out_meta)) = (fs::metadata(script), fs::metadata(&out_so)) {
				src_meta
					.modified()
					.unwrap_or(std::time::SystemTime::UNIX_EPOCH)
					> out_meta
						.modified()
						.unwrap_or(std::time::SystemTime::UNIX_EPOCH)
			} else {
				true
			}
		};

		if needs_compile {
			eprintln!("{:>12} {}", "Compiling".green().bold(), script.display());
			mochi_script::qw::compile_qw_script(script, &out_so)
				.map_err(|e| Error::New | format!("failed to compile {}: {e}", script.display()))?;
			compiled_count += 1;
		} else if info.verbose > 0 {
			eprintln!("  {} {} (fresh)", "·".dimmed(), script.display());
		}
	}

	let elapsed = start_time.elapsed();
	eprintln!(
		"{:>12} target(s) [{} scripts compiled] in {:.2?}",
		"Finished".green().bold(),
		compiled_count,
		elapsed
	);

	Ok(())
}

fn collect_qw_files(dir: &Path, out: &mut Vec<PathBuf>) {
	if dir.is_file() {
		if dir.extension().is_some_and(|ext| ext == "qw") {
			out.push(dir.to_path_buf());
		}
		return;
	}

	if let Ok(entries) = fs::read_dir(dir) {
		for entry in entries.flatten() {
			let path = entry.path();
			if path.is_dir() {
				let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
				if name != "target" && name != ".git" && name != "build" && name != "crates" {
					collect_qw_files(&path, out);
				}
			} else if path.extension().is_some_and(|ext| ext == "qw") {
				out.push(path);
			}
		}
	}
}
