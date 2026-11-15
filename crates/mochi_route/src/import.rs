/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::path::Path;
use mochi_asset::ModelAsset;
use owo_colors::OwoColorize;

use crate::Error;


pub struct ImportInfo<'a> {
	pub input: &'a Path,
	pub output: Option<&'a Path>,
	pub binary: bool,
}

pub fn import(info: ImportInfo) -> Result<(), Error> {
	if !info.input.exists() {
		return Err(Error::New | format!("input file does not exist: {}", info.input.display()));
	}

	let ext = info.input.extension().and_then(|s| s.to_str()).unwrap_or("");
	if ext != "glb" && ext != "gltf" {
		return Err(Error::New | format!("unsupported 3d format '.{ext}', expected .glb or .gltf"));
	}

	eprintln!("{:>12} {}", "Importing".green().bold(), info.input.display());

	let model = ModelAsset::load_gltf(info.input)
		.map_err(|e| Error::New | format!("failed to import {}: {e}", info.input.display()))?;

	let out_path = if let Some(out) = info.output {
		out.to_path_buf()
	} else {
		let stem = info
			.input
			.file_stem()
			.and_then(|s| s.to_str())
			.unwrap_or("model");
		let target_ext = if info.binary { "mmodel.bin" } else { "mmodel.ron" };
		info.input.with_file_name(format!("{stem}.{target_ext}"))
	};

	let out_str = out_path.to_string_lossy();
	if info.binary {
		model
			.save_binary(&out_str)
			.map_err(|e| Error::New | format!("failed to save binary model: {e}"))?;
	} else {
		model
			.save_ron(&out_str)
			.map_err(|e| Error::New | format!("failed to save model: {e}"))?;
	}

	eprintln!(
		"{:>12} {} ({} submeshes, {} materials)",
		"Finished".green().bold(),
		out_path.display(),
		model.submeshes.len(),
		model.materials.len()
	);

	Ok(())
}
