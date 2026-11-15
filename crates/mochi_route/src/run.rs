/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::path::Path;
use mochi_audio::AudioClip;
use mochi_core::{Engine, ProjectConfig};
use mochi_rendering::AntiAliasingMode;
use owo_colors::OwoColorize;

use crate::Error;


pub struct RunInfo<'a> {
	pub path: &'a Path,
	pub scene: Option<&'a Path>,
	pub config: Option<&'a Path>,
	pub width: Option<u32>,
	pub height: Option<u32>,
	pub title: Option<String>,
	pub vsync: Option<bool>,
	pub aa: Option<String>,
	pub msaa: Option<u32>,
	pub mute: bool,
}

pub fn run(info: RunInfo) -> Result<(), Error> {
	let project_dir = match ProjectConfig::find_project_root(info.path) {
		Some(dir) => dir,
		None => {
			return Err(Error::New | format!("cannot find mochi.ron in '{}'", info.path.display()));
		}
	};

	let config_file = if let Some(cfg) = info.config {
		cfg.to_path_buf()
	} else {
		project_dir.join("mochi.ron")
	};

	if !config_file.exists() {
		return Err(Error::New | format!("cannot find mochi.ron in '{}'", project_dir.display()));
	}

	let mut config = ProjectConfig::load(&config_file)
		.map_err(|e| Error::New | e)?;

	// If a specific scene is passed (via info.scene or info.path being a .mscene file)
	if let Some(scene) = info.scene {
		config.scene.start_scene = scene.to_string_lossy().to_string();
	} else if info.path.is_file() && info.path.file_name().map_or(true, |n| n != "mochi.ron") {
		config.scene.start_scene = info.path.to_string_lossy().to_string();
	}

	// Verify that the start scene actually exists before starting engine or creating window
	let resolved_scene = config.resolve_path(&config.scene.start_scene);
	if !resolved_scene.exists() {
		return Err(Error::New | format!("start scene does not exist: {}", resolved_scene.display()));
	}

	if let Some(w) = info.width {
		config.window.width = w;
	}
	if let Some(h) = info.height {
		config.window.height = h;
	}
	if let Some(t) = info.title {
		config.window.title = t;
	}
	if let Some(v) = info.vsync {
		config.window.vsync = v;
	}

	if let Some(aa_str) = &info.aa {
		let mode = parse_aa_mode(aa_str);
		mochi_script::api::mochi_set_aa_mode(mode.to_i32());
	}

	if let Some(msaa) = info.msaa {
		mochi_script::api::mochi_set_msaa(msaa as i32);
	}

	// QW style execution message
	eprintln!("{:>12} {}", "Running".green().bold(), config.scene.start_scene);

	let mut engine = Engine::with_config(config);

	if !info.mute {
		if let Some(audio) = engine.audio_system() {
			let startup_chime = std::sync::Arc::new(AudioClip::sine_wave(523.25, 0.35, 44100));
			audio.play_one_shot(startup_chime, 0.15);
		}
	}

	engine.set_idle(|_dt| {});

	if let Err(err) = engine.run() {
		return Err(Error::New | format!("runtime error: {err}"));
	}

	Ok(())
}

pub fn parse_aa_mode(s: &str) -> AntiAliasingMode {
	match s.to_ascii_lowercase().replace('_', "-").as_str() {
		"off" | "none" => AntiAliasingMode::Off,
		"msaa-2x" | "msaa2x" => AntiAliasingMode::Msaa2x,
		"msaa-4x" | "msaa4x" => AntiAliasingMode::Msaa4x,
		"msaa-8x" | "msaa8x" => AntiAliasingMode::Msaa8x,
		"fxaa" => AntiAliasingMode::Fxaa,
		"smaa" | "smaa-1x" | "smaa1x" => AntiAliasingMode::Smaa1x,
		"smaa-t2x" | "smaat2x" => AntiAliasingMode::SmaaT2x,
		"smaa-s2x" | "smaas2x" => AntiAliasingMode::SmaaS2x,
		"smaa-4x" | "smaa4x" => AntiAliasingMode::Smaa4x,
		"taa" => AntiAliasingMode::Taa,
		"fsr-native" | "fsr-native-aa" => AntiAliasingMode::FsrNativeAa,
		"fsr" | "fsr-quality" => AntiAliasingMode::FsrQuality,
		"fsr-balanced" => AntiAliasingMode::FsrBalanced,
		"fsr-performance" => AntiAliasingMode::FsrPerformance,
		"fsr-ultra-performance" => AntiAliasingMode::FsrUltraPerformance,
		other => {
			eprintln!(
				"{}{} unknown anti-aliasing mode '{}', falling back to taa",
				"warning".yellow().bold(),
				":".bright_black(),
				other
			);
			AntiAliasingMode::Taa
		}
	}
}
