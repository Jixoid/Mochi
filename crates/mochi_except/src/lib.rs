/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::io::Write;
use std::panic::Location;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;
use std::time::Instant;


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
	if module_path.starts_with("mochi_rhi") {
		"MCH.RHI"
	} else if module_path.starts_with("mochi_ahi") {
		"MCH.AHI"
	} else if module_path.starts_with("mochi_math") {
		"MCH.MATH"
	} else if module_path.starts_with("mochi_plugin") {
		"MCH.PLUGIN"
	} else if module_path.starts_with("mochi_except") {
		"MCH.EXCEPT"
	} else if module_path.starts_with("Mochi::audio") || module_path.starts_with("mochi::audio") {
		"MCH.AUDIO"
	} else if module_path.starts_with("Mochi::render") || module_path.starts_with("mochi::render") {
		"MCH.RENDER"
	} else if module_path.starts_with("Mochi::display") || module_path.starts_with("mochi::display") {
		"MCH.DISP"
	} else if module_path.starts_with("Mochi::core") || module_path.starts_with("mochi::core") {
		"MCH.SYS"
	} else if module_path.starts_with("Mochi") || module_path.starts_with("mochi") {
		"MCH.SYS"
	} else {
		"MCH.LOG"
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
	let file_name = std::path::Path::new(loc.file())
		.file_name()
		.and_then(|s| s.to_str())
		.unwrap_or(loc.file());

	let (sym, col) = match typ {
		LogType::Verbose => ('V', "\x1b[1;30m"),
		LogType::Log     => ('L', "\x1b[1;30m"),
		LogType::Display => ('D', "\x1b[1;37m"),
		LogType::Warning => ('W', "\x1b[1;33m"),
		LogType::Error   => ('E', "\x1b[1;31m"),
		LogType::Panic   => ('P', "\x1b[1;31m"),
	};

	let stderr = std::io::stderr();
	let mut lock = stderr.lock();

	if (typ as u8) >= (LogType::Warning as u8) {
		let _ = writeln!(
			lock,
			"{col}{sym} [{sec}.{ms:03}] @{tag} {args}\n\
			 ├─> \x1b[1;30mmodule: {module_path}\n{col}\
			 ╰─> \x1b[1;30mfile: {file_name}:{line}:{column}\x1b[0m",
			col = col,
			sym = sym,
			sec = sec,
			ms = ms,
			tag = tag,
			args = args,
			module_path = module_path,
			file_name = file_name,
			line = loc.line(),
			column = loc.column()
		);
	} else {
		let _ = writeln!(
			lock,
			"{col}{sym} [{sec}.{ms:03}] @{tag} {args}\x1b[0m",
			col = col,
			sym = sym,
			sec = sec,
			ms = ms,
			tag = tag,
			args = args,
		);
	}
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
