/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;
use mochi_rhi::{self as rhi, Synchronizer};
use crate::audio::AudioSystem;
use crate::display::Display;
use crate::rendering::RenderingSystem;


pub struct EngineApp {
	title: String,
	width: u32,
	height: u32,
	last_frame_time: Option<Instant>,
	fps_timer: Instant,
	fps_counter: u32,
	frame_count: u64,
	idle_cb: Option<Box<dyn FnMut(f32)>>,

	device: Option<rhi::Device>,
	alloca: Option<rhi::Allocator>,
	cmdmng: Option<rhi::CommandManager>,
	syncer: Option<rhi::Synchronizer>,

	rendering_system: Option<RenderingSystem>,
	audio_system: Option<AudioSystem>,
	display: Option<Display>,
}

impl EngineApp {
	pub fn new(title: &str, width: u32, height: u32) -> Self {
		// RHI
		let device = rhi::Device::new("MochiEngine", 0, 1, 0, 0);
		let alloca = rhi::Allocator::new(&device);
		let cmdmng = rhi::CommandManager::new(&device);
		let syncer = Synchronizer::new(&device, RenderingSystem::MAX_FRAMES_IN_FLIGHT);

		// Audio
		let audio_system = AudioSystem::new("Mochi Audio", 0, 1, 0, 0);

		// SYS
		let rendering_system = RenderingSystem::new(&cmdmng).expect("failed to create RenderingSystem");

		Self {
			title: title.to_string(),
			width,
			height,
			
			device: Some(device),
			alloca: Some(alloca),
			cmdmng: Some(cmdmng),
			syncer: Some(syncer),

			rendering_system: Some(rendering_system),
			audio_system: Some(audio_system),
			display: None,
			last_frame_time: None,
			fps_timer: Instant::now(),
			fps_counter: 0,
			frame_count: 0,
			idle_cb: None,
		}
	}


	pub fn set_idle<F: FnMut(f32) + 'static>(&mut self, cb: F) {
		self.idle_cb = Some(Box::new(cb));
	}


	pub fn device(&self) -> &rhi::Device {
		self.device.as_ref().unwrap()
	}

	pub fn alloca(&self) -> &rhi::Allocator {
		self.alloca.as_ref().unwrap()
	}

	pub fn audio_system(&self) -> Option<&AudioSystem> {
		self.audio_system.as_ref()
	}


	pub fn display(&self) -> Option<&Display> {
		self.display.as_ref()
	}
}


impl ApplicationHandler for EngineApp {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.display.is_none() {
			match Display::new(event_loop, self.device.as_ref().unwrap(), &self.title, self.width, self.height) {
				Ok(display) => {
					if let (Some(rendering), Some(target)) = (&mut self.rendering_system, display.render_target(0)) {
						rendering.init_pipeline(
							self.device.as_ref().unwrap(),
							self.alloca.as_ref().unwrap(),
							target.color_format(),
							target.depth_format(),
						);
					}
					display.request_redraw();
					self.display = Some(display);
					println!("[Engine] Display and Vulkan swapchain attached successfully.");
				}
				Err(err) => {
					eprintln!("[Engine] Failed to initialize display: {err}");
					event_loop.exit();
				}
			}
		}
	}

	fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
		match event {
			WindowEvent::CloseRequested => {
				println!("[Engine] Window close requested, shutting down...");
				event_loop.exit();
			}
			WindowEvent::KeyboardInput {
				event:
					KeyEvent {
						physical_key: PhysicalKey::Code(KeyCode::Escape),
						state: ElementState::Pressed,
						..
					},
				..
			} => {
				println!("[Engine] Escape pressed, shutting down...");
				event_loop.exit();
			}
			WindowEvent::Resized(physical_size) => {
				if let Some(display) = &mut self.display {
					display.resize(physical_size.width, physical_size.height);
					display.request_redraw();
				}
			}
			WindowEvent::RedrawRequested => {
				let now = Instant::now();
				let dt = self.last_frame_time.map_or(0.016, |t| (now - t).as_secs_f32());
				self.last_frame_time = Some(now);

				if let Some(cb) = &mut self.idle_cb {
					cb(dt);
				}

				self.fps_counter += 1;
				if self.fps_timer.elapsed().as_secs() >= 1 {
					let fps = self.fps_counter;
					self.fps_counter = 0;
					self.fps_timer = Instant::now();
					if let Some(display) = &self.display {
						display.window().set_title(&format!(
							"{} - [{} FPS] Frame: {}",
							self.title, fps, self.frame_count
						));
					}
				}

				if let Some(audio) = &self.audio_system {
					audio.update();
				}

				if let (Some(display), Some(rendering), Some(sync), Some(cmdmng)) = (
					&mut self.display,
					&mut self.rendering_system,
					&mut self.syncer,
					&self.cmdmng,
				) {
					rendering.render(sync, cmdmng, display, [0.1, 0.15, 0.25, 1.0]);

					self.frame_count += 1;

					if let Ok(limit_str) = std::env::var("MOCHI_TEST_EXIT_FRAMES") {
						if let Ok(limit) = limit_str.parse::<u64>() {
							if self.frame_count >= limit {
								println!("[Engine] Reached test frame limit ({limit}), exiting...");
								event_loop.exit();
								return;
							}
						}
					}

					display.request_redraw();
				}
			}
			_ => {}
		}
	}

	fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
		println!("[Engine] Cleaning up Engine resources...");
		self.audio_system = None;

		self.device.as_ref().unwrap().wait_idle();
		
		self.rendering_system = None;
		self.display = None;

		self.syncer = None;
		self.cmdmng = None;
		self.alloca = None;
		self.device = None;

		println!("[Engine] Cleanup completed cleanly.");
	}
}


pub struct Engine {
	app: EngineApp,
}

impl Engine {
	pub fn new(title: &str, width: u32, height: u32) -> Self {
		Self {
			app: EngineApp::new(title, width, height),
		}
	}

	pub fn set_idle<F: FnMut(f32) + 'static>(&mut self, cb: F) {
		self.app.set_idle(cb);
	}

	pub fn audio_system(&self) -> Option<&AudioSystem> {
		self.app.audio_system()
	}

	pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
		let event_loop = EventLoop::new()?;
		event_loop.set_control_flow(ControlFlow::Poll);
		event_loop.run_app(&mut self.app)?;
		Ok(())
	}
}
