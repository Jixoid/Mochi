/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::path::PathBuf;
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;

pub mod build;
pub mod check;
pub mod clean;
pub mod error;
pub mod import;
pub mod init;
pub mod run;

pub use error::Error;


pub fn route() {
	mochi_except::start();
	let cli = Cli::parse();
	main_cmd(cli);
}


#[derive(Parser, Debug)]
#[command(name = "mochi", version, about = "mochi 3d game engine")]
pub struct Cli {
	#[command(subcommand)]
	pub command: MainCommands,
}

#[derive(Subcommand, Debug)]
pub enum MainCommands {
	/// Run the game engine
	Run {
		/// Directory or scene to run
		#[arg(default_value = ".")]
		path: PathBuf,

		/// Specific scene file to run
		#[arg(short, long)]
		scene: Option<PathBuf>,

		/// Project configuration file
		#[arg(short, long)]
		config: Option<PathBuf>,

		/// Window width
		#[arg(long)]
		width: Option<u32>,

		/// Window height
		#[arg(long)]
		height: Option<u32>,

		/// Window title
		#[arg(long)]
		title: Option<String>,

		/// Enable VSync
		#[arg(long, conflicts_with = "no_vsync")]
		vsync: bool,

		/// Disable VSync
		#[arg(long = "no-vsync", conflicts_with = "vsync")]
		no_vsync: bool,

		/// Anti-aliasing mode
		#[arg(long)]
		aa: Option<String>,

		/// MSAA samples
		#[arg(long)]
		msaa: Option<u32>,

		/// Enable FSR 3.1 Frame Generation (2x display present)
		#[arg(long = "fg", alias = "frame-gen", conflicts_with = "no_fg")]
		fg: bool,

		/// Disable FSR 3.1 Frame Generation
		#[arg(long = "no-fg", alias = "no-frame-gen", conflicts_with = "fg")]
		no_fg: bool,

		/// Mute audio
		#[arg(short, long)]
		mute: bool,
	},

	/// Compile the package and QW scripts
	Build {
		/// Directory of the package to build
		#[arg(default_value = ".")]
		path: PathBuf,

		/// Force recompilation
		#[arg(short, long)]
		force: bool,

		/// Verbose output
		#[arg(short, long, action = clap::ArgAction::Count)]
		verbose: u8,
	},

	/// Check the package for errors
	Check {
		/// Directory to check
		#[arg(default_value = ".")]
		path: PathBuf,
	},

	/// Clean build and script cache
	Clean {
		/// Directory to clean
		#[arg(default_value = ".")]
		path: PathBuf,
	},

	/// Create a new mochi project
	Init {
		/// Directory to initialize
		#[arg(default_value = ".")]
		path: PathBuf,

		/// Set the project name
		#[arg(short, long)]
		name: Option<String>,

		/// Set the project description
		#[arg(short, long)]
		desc: Option<String>,

		/// Do not initialize a new git repository
		#[arg(long)]
		no_git: bool,

		/// Force init overwrite
		#[arg(short, long)]
		force: bool,
	},

	/// Import 3D assets (GLTF/GLB) into Mochi model format
	Import {
		/// Input GLTF/GLB file path
		input: PathBuf,

		/// Output model file path (.mmodel.ron or .mmodel.bin)
		#[arg(short, long)]
		output: Option<PathBuf>,

		/// Save as binary postcard format (.mmodel.bin)
		#[arg(short, long)]
		binary: bool,
	},
}

fn main_cmd(cli: Cli) {
	let res = match cli.command {
		MainCommands::Run {
			path,
			scene,
			config,
			width,
			height,
			title,
			vsync,
			no_vsync,
			aa,
			msaa,
			fg,
			no_fg,
			mute,
		} => run::run(run::RunInfo {
			path: &path,
			scene: scene.as_deref(),
			config: config.as_deref(),
			width,
			height,
			title,
			vsync: if vsync {
				Some(true)
			} else if no_vsync {
				Some(false)
			} else {
				None
			},
			aa,
			msaa,
			frame_gen: fg,
			no_frame_gen: no_fg,
			mute,
		}),

		MainCommands::Build {
			path,
			force,
			verbose,
		} => build::build(build::BuildInfo {
			path: &path,
			force,
			verbose,
		}),

		MainCommands::Check { path } => check::check(check::CheckInfo { path: &path }),

		MainCommands::Clean { path } => clean::clean(clean::CleanInfo { path: &path }),

		MainCommands::Init {
			path,
			name,
			desc,
			no_git,
			force,
		} => init::init(init::InitInfo {
			path: path.to_string_lossy().to_string(),
			name: name.unwrap_or_default(),
			desc: desc.unwrap_or_default(),
			no_git,
			force,
		}),

		MainCommands::Import {
			input,
			output,
			binary,
		} => import::import(import::ImportInfo {
			input: &input,
			output: output.as_deref(),
			binary,
		}),
	};

	if let Err(e) = res {
		eprintln!("{}{} {}", "error".red().bold(), ":".bright_black(), e);
		std::process::exit(1);
	}
}
