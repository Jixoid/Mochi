/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::io::Write;
use std::panic::Location;
use std::path::Path;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

use owo_colors::OwoColorize;


#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogType {
	Verbose = 0,
	Log = 1,
	Display = 2,
	Warning = 3,
	Error = 4,
	Panic = 5,
}

static BOOT: OnceLock<Instant> = OnceLock::new();

static GLOBAL_LEVEL: AtomicU8 = AtomicU8::new(
	if cfg!(debug_assertions) {
		LogType::Verbose as u8
	} else {
		LogType::Display as u8
	}
);

pub fn start() {
	BOOT.get_or_init(Instant::now);
}

pub fn set_log_level(level: LogType) {
	GLOBAL_LEVEL.store(level as u8, Ordering::Relaxed);
}

pub fn get_log_level() -> LogType {
	match GLOBAL_LEVEL.load(Ordering::Relaxed) {
		0 => LogType::Verbose,
		1 => LogType::Log,
		2 => LogType::Display,
		3 => LogType::Warning,
		4 => LogType::Error,
		_ => LogType::Panic,
	}
}

pub fn resolve_module_tag(module_path: &str) -> &'static str {
	match () {
		_ if module_path.starts_with("mochi_rhi") => "RHI",
		_ if module_path.starts_with("mochi_ahi") => "AHI",

		_ if module_path.starts_with("mochi_math")   => "MATH",
		_ if module_path.starts_with("mochi_plugin") => "PLUGIN",
		_ if module_path.starts_with("mochi_except") => "EXCEPT",
		
		_ if module_path.starts_with("Mochi::render") || module_path.starts_with("mochi::render")   => "RENDER",
		_ if module_path.starts_with("Mochi::audio") || module_path.starts_with("mochi::audio")     => "AUDIO",
		_ if module_path.starts_with("Mochi::display") || module_path.starts_with("mochi::display") => "DISPLAY",
		
		_ if module_path.starts_with("Mochi::core") || module_path.starts_with("mochi::core") => "CORE",

		_ => "MOCHI",
	}
}

#[track_caller]
pub fn log(typ: LogType, module_path: &'static str, args: std::fmt::Arguments) {
	if (typ as u8) < GLOBAL_LEVEL.load(Ordering::Relaxed) {
		return;
	}

	let loc = Location::caller();
	let boot = BOOT.get_or_init(Instant::now);
	let elapsed = boot.elapsed();
	let total_ms = elapsed.as_millis();
	let sec = total_ms / 1000;
	let ms = total_ms % 1000;

	let tag = resolve_module_tag(module_path);
	let file_name = Path::new(loc.file())
		.file_name()
		.and_then(|s| s.to_str())
		.unwrap_or(loc.file());

	let sym = match typ {
		LogType::Verbose => 'V',
		LogType::Log     => 'L',
		LogType::Display => 'D',
		LogType::Warning => 'W',
		LogType::Error   => 'E',
		LogType::Panic   => 'P',
	};

	let msg = if (typ as u8) >= (LogType::Warning as u8) {
		format!(
			"{sym} [{sec}.{ms:03}] @{tag} {args}\n\
			├─> mod: {module_path}\n\
			╰─> file: {file_name}:{line}:{column}",
			
			line = loc.line(),
			column = loc.column(),
		)
	} else {
		format!("{sym} [{sec}.{ms:03}] @{tag} {args}")
	};

	let stderr = std::io::stderr();
	let mut lock = stderr.lock();

	let msg = match typ {
		LogType::Log | LogType::Verbose => msg.bold().bright_black().to_string(),
		LogType::Display => msg.to_string(),
		LogType::Warning => msg.bold().yellow().to_string(),
		LogType::Error | LogType::Panic => msg.bold().red().to_string(),
	};

	writeln!(lock, "{msg}").unwrap()
}

#[track_caller]
pub fn log_panic(module_path: &'static str, args: std::fmt::Arguments) -> ! {
	log(LogType::Panic, module_path, args);
	panic!("{args}");
}


#[macro_export]
macro_rules! mochi_panic {
	($($arg:tt)+) => {
		$crate::log_panic(module_path!(), format_args!($($arg)+))
	};
}

#[macro_export]
macro_rules! mochi_error {
	($($arg:tt)+) => {
		$crate::log($crate::LogType::Error, module_path!(), format_args!($($arg)+))
	};
}

#[macro_export]
macro_rules! mochi_warn {
	($($arg:tt)+) => {
		$crate::log($crate::LogType::Warning, module_path!(), format_args!($($arg)+))
	};
}

#[macro_export]
macro_rules! mochi_disp {
	($($arg:tt)+) => {
		$crate::log($crate::LogType::Display, module_path!(), format_args!($($arg)+))
	};
}

#[macro_export]
macro_rules! mochi_log {
	($($arg:tt)+) => {
		$crate::log($crate::LogType::Log, module_path!(), format_args!($($arg)+))
	};
}

#[macro_export]
macro_rules! mochi_verb {
	($($arg:tt)+) => {
		if cfg!(debug_assertions) {
			$crate::log($crate::LogType::Verbose, module_path!(), format_args!($($arg)+))
		}
	};
}
