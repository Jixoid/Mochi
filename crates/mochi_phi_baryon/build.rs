use std::path::Path;


fn main() {
	println!("cargo:rerun-if-changed=cpp/baryon_c_api.cpp");
	println!("cargo:rerun-if-changed=cpp/baryon_c_api.h");

	let baryon_root = Path::new("/home/alforce/ExtProj/Baryon");
	let baryon_include = baryon_root.join("baryon/include");
	let entt_include = baryon_root.join("build/_deps/entt_fetch-src/src");
	let baryon_lib_dir = baryon_root.join("build/baryon");

	cc::Build::new()
		.cpp(true)
		.std("c++23")
		.file("cpp/baryon_c_api.cpp")
		.include("cpp")
		.include(&baryon_include)
		.include(&entt_include)
		.compile("baryon_c_bridge");

	println!(
		"cargo:rustc-link-search=native={}",
		baryon_lib_dir.display()
	);
	println!("cargo:rustc-link-lib=static=Baryon");
	println!("cargo:rustc-link-lib=stdc++");
}
