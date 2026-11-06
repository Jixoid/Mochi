/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/

use std::path::PathBuf;
use std::process::Command;

fn main() {
	let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
	let build_dir = manifest_dir.join("build");

	// Automatically invoke ninja to compile drivers if needed
	if build_dir.exists() {
		let _ = Command::new("ninja")
			.args([
				"-C",
				build_dir.to_str().unwrap(),
				"qaos_mochi_graphic_driver",
				"qaos_mochi_audio_driver",
				"qaos_mochi_driver_common",
				"miniaudio",
			])
			.status();
	}

	let vulkan_driver_dir = build_dir.join("drivers/vulkan");
	let audio_driver_dir = build_dir.join("drivers/miniaudio");
	let driver_common_dir = build_dir.join("drivers");
	let miniaudio_dir = build_dir.join("thirdparty/miniaudio");

	println!(
		"cargo:rustc-link-search=native={}",
		vulkan_driver_dir.display()
	);
	println!(
		"cargo:rustc-link-search=native={}",
		audio_driver_dir.display()
	);
	println!(
		"cargo:rustc-link-search=native={}",
		driver_common_dir.display()
	);
	println!("cargo:rustc-link-search=native={}", miniaudio_dir.display());

	println!("cargo:rustc-link-lib=static=qaos_mochi_graphic_driver");
	println!("cargo:rustc-link-lib=static=qaos_mochi_audio_driver");
	println!("cargo:rustc-link-lib=static=qaos_mochi_driver_common");

	// Native & thirdparty dependencies
	println!("cargo:rustc-link-lib=miniaudio");
	println!("cargo:rustc-link-lib=glfw");
	println!("cargo:rustc-link-lib=vulkan");
	println!("cargo:rustc-link-lib=shaderc_shared");
	println!("cargo:rustc-link-lib=stdc++");

	println!(
		"cargo:rustc-link-arg=-Wl,-rpath,{}",
		miniaudio_dir.display()
	);

	// Clang Sanitizer runtimes if C++ is built in Debug with ASan/UBSan
	if let Ok(out) = Command::new("clang++")
		.arg("-print-file-name=libclang_rt.asan-x86_64.a")
		.output()
	{
		let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
		let path = PathBuf::from(&path_str);
		if path.exists() {
			if let Some(parent) = path.parent() {
				println!("cargo:rustc-link-search=native={}", parent.display());
				println!("cargo:rustc-link-lib=static=clang_rt.asan-x86_64");
				println!("cargo:rustc-link-lib=static=clang_rt.ubsan_standalone-x86_64");
			}
		}
	}

	println!("cargo:rerun-if-changed=drivers/include/mochi/driver_abi.h");
	println!("cargo:rerun-if-changed=drivers/vulkan/VKc_api.cc");
	println!("cargo:rerun-if-changed=drivers/vulkan/manager/VKswapchain_manager.cc");
	println!("cargo:rerun-if-changed=drivers/miniaudio/MAc_api.cc");
	println!("cargo:rerun-if-changed=build/drivers/vulkan/libqaos_mochi_graphic_driver.a");
}
