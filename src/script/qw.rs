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
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;
use libloading::Library;
use mochi_except::*;


static REVISION_COUNTER: AtomicU64 = AtomicU64::new(1);

type OnStartFn = unsafe extern "C" fn(u64);
type OnUpdateFn = unsafe extern "C" fn(u64, f32);

pub struct LoadedQwScript {
	pub source_path: PathBuf,
	pub last_mtime: SystemTime,
	_lib: Option<Library>,
	on_start: Option<OnStartFn>,
	on_update: Option<OnUpdateFn>,
}

impl LoadedQwScript {
	pub fn empty(source_path: &Path) -> Self {
		Self {
			source_path: source_path.to_path_buf(),
			last_mtime: SystemTime::UNIX_EPOCH,
			_lib: None,
			on_start: None,
			on_update: None,
		}
	}
	pub fn load(source_path: &Path) -> Result<Self, String> {
		let metadata = fs::metadata(source_path)
			.map_err(|e| format!("Failed to read metadata for {}: {e}", source_path.display()))?;
		let mtime = metadata.modified()
			.map_err(|e| format!("Failed to get mtime for {}: {e}", source_path.display()))?;

		let cache_dir = PathBuf::from("target/qw_cache");
		fs::create_dir_all(&cache_dir)
			.map_err(|e| format!("Failed to create cache dir {}: {e}", cache_dir.display()))?;

		let stem = source_path.file_stem().and_then(|s| s.to_str()).unwrap_or("script");
		let rev = REVISION_COUNTER.fetch_add(1, Ordering::SeqCst);
		let target_so = cache_dir.join(format!("{stem}_v{rev}.so"));

		compile_qw_script(source_path, &target_so)?;

		let lib = unsafe {
			Library::new(&target_so)
				.map_err(|e| format!("Failed to load dynamic library {}: {e}", target_so.display()))?
		};

		let on_start = unsafe {
			lib.get::<OnStartFn>(b"on_start").ok().map(|s| *s)
		};

		let on_update = unsafe {
			lib.get::<OnUpdateFn>(b"on_update").ok().map(|s| *s)
		};

		mochi_disp!(
			"Loaded QW script '{}' (on_start: {}, on_update: {})",
			source_path.display(),
			on_start.is_some(),
			on_update.is_some()
		);

		Ok(Self {
			source_path: source_path.to_path_buf(),
			last_mtime: mtime,
			_lib: Some(lib),
			on_start,
			on_update,
		})
	}

	pub fn reload_if_changed(&mut self) -> Result<bool, String> {
		let metadata = match fs::metadata(&self.source_path) {
			Ok(m) => m,
			Err(e) => return Err(format!("Failed to read metadata: {e}")),
		};

		let current_mtime = match metadata.modified() {
			Ok(t) => t,
			Err(e) => return Err(format!("Failed to get mtime: {e}")),
		};

		if current_mtime <= self.last_mtime {
			return Ok(false);
		}

		mochi_log!("Detected change in '{}', recompiling...", self.source_path.display());
		let reloaded = Self::load(&self.source_path)?;
		*self = reloaded;
		Ok(true)
	}

	pub fn call_start(&self, entity_id: u64) {
		if let Some(on_start) = self.on_start {
			unsafe {
				on_start(entity_id);
			}
		}
	}

	pub fn call_update(&self, entity_id: u64, dt: f32) {
		if let Some(on_update) = self.on_update {
			unsafe {
				on_update(entity_id, dt);
			}
		}
	}
}

static DEFAULT_MOCHI_QW: &str = include_str!("mochi.qw");


pub fn ensure_engine_unit() -> Result<PathBuf, String> {
	let dep_dir = PathBuf::from("target/qw_cache/mochi");
	let qwu_file = dep_dir.join("out.qwu");
	let obj_file = dep_dir.join("out.o");

	let source_path = if Path::new("src/script/mochi.qw").exists() {
		PathBuf::from("src/script/mochi.qw")
	} else {
		fs::create_dir_all(&dep_dir)
			.map_err(|e| format!("Failed to create {}: {e}", dep_dir.display()))?;
		let fallback = dep_dir.join("mochi.qw");
		if !fallback.exists() {
			fs::write(&fallback, DEFAULT_MOCHI_QW)
				.map_err(|e| format!("Failed to write fallback mochi.qw: {e}"))?;
		}
		fallback
	};

	let needs_compile = !qwu_file.exists() || !obj_file.exists() || {
		if let (Ok(src_meta), Ok(qwu_meta)) = (fs::metadata(&source_path), fs::metadata(&qwu_file)) {
			src_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH) > qwu_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH)
		} else {
			true
		}
	};

	if needs_compile {
		fs::create_dir_all(&dep_dir)
			.map_err(|e| format!("Failed to create {}: {e}", dep_dir.display()))?;

		let output = Command::new("qw.debug")
			.arg("compile")
			.arg("--path")
			.arg(&source_path)
			.arg("--outdir")
			.arg(&dep_dir)
			.arg("--link")
			.arg("shared")
			.output()
			.map_err(|e| format!("Failed to execute qw.debug: {e}. Is qw.debug in PATH?"))?;

		if !output.status.success() {
			let stderr = String::from_utf8_lossy(&output.stderr);
			let stdout = String::from_utf8_lossy(&output.stdout);
			return Err(format!("Failed to compile engine mochi unit:\n{stderr}\n{stdout}"));
		}
	}

	let canonical = fs::canonicalize(&dep_dir).unwrap_or(dep_dir);
	Ok(canonical)
}

fn compile_qw_script(source_path: &Path, output_so_path: &Path) -> Result<(), String> {
	let mochi_dep_dir = ensure_engine_unit()?;

	let output = Command::new("qw.debug")
		.arg("compile")
		.arg("--path")
		.arg(source_path)
		.arg("--dep")
		.arg(format!("mochi={}", mochi_dep_dir.display()))
		.arg("-o")
		.arg(output_so_path)
		.arg("--link")
		.arg("shared")
		.output()
		.map_err(|e| format!("Failed to execute qw.debug: {e}. Is qw.debug in PATH?"))?;

	if !output.status.success() {
		let stderr = String::from_utf8_lossy(&output.stderr);
		let stdout = String::from_utf8_lossy(&output.stdout);
		return Err(format!("QW compilation failed for {}:\n{stderr}\n{stdout}", source_path.display()));
	}

	if !output_so_path.exists() {
		return Err(format!("QW compilation finished but {} was not created", output_so_path.display()));
	}

	Ok(())
}
