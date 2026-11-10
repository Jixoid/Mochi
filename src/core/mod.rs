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
use crate::audio::AudioSystem;
use crate::display::Display;
use crate::rendering::RenderingSystem;
use crate::rhi as rhi;
use crate::scene::{CameraComponent, DirectionalLightComponent, Mesh, MeshRendererComponent, PointLightComponent, Scene, TransformComponent};
use mochi_except::*;
use mochi_math::Vec3;


pub struct EngineApp {
	title: String,
	width: u32,
	height: u32,
	start_time: Instant,
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
	scene: Scene,
}

impl EngineApp {
	pub fn new(title: &str, width: u32, height: u32) -> Self {
		// RHI
		let device = rhi::Device::new("MochiEngine", 0, 1, 0, 0);
		let alloca = rhi::Allocator::new(&device);
		let cmdmng = rhi::CommandManager::new(&device);
		let syncer = rhi::Synchronizer::new(&device, RenderingSystem::MAX_FRAMES_IN_FLIGHT);

		// Audio
		let audio_system = AudioSystem::new("Mochi Audio", 0, 1, 0, 0);

		// SYS
		let rendering_system = RenderingSystem::new(&cmdmng).expect("failed to create RenderingSystem");

		// Initialize default scene environment
		let mut scene = Scene::new();

		let mut cam_transform = TransformComponent::default();
		cam_transform.translation = Vec3::new(0.0, 1.2, 3.5);
		scene.spawn((
			cam_transform,
			CameraComponent::new(60.0, 0.1, 1000.0),
		));

		let sun_transform = TransformComponent::default();
		scene.spawn((
			sun_transform,
			DirectionalLightComponent {
				color: Vec3::new(1.0, 0.95, 0.9),
				intensity: 1.5,
				direction: Vec3::new(0.5, -1.0, 0.5).normalize(),
				cast_shadows: true,
			},
		));

		let mut pt_transform = TransformComponent::default();
		pt_transform.translation = Vec3::new(-1.5, 1.0, 1.0);
		scene.spawn((
			pt_transform,
			PointLightComponent {
				color: Vec3::new(1.0, 0.6, 0.2),
				intensity: 3.0,
				radius: 8.0,
			},
		));

		Self {
			title: title.to_string(),
			width,
			height,
			start_time: Instant::now(),
			device: Some(device),
			alloca: Some(alloca),
			cmdmng: Some(cmdmng),
			syncer: Some(syncer),

			rendering_system: Some(rendering_system),
			audio_system: Some(audio_system),
			display: None,
			scene,
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

	pub fn scene(&self) -> &Scene {
		&self.scene
	}

	pub fn scene_mut(&mut self) -> &mut Scene {
		&mut self.scene
	}
}

impl ApplicationHandler for EngineApp {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.display.is_none() {
			match Display::new(event_loop, self.device.as_ref().unwrap(), &self.title, self.width, self.height) {
				Ok(display) => {
					if let (Some(rendering), Some(target)) = (&mut self.rendering_system, display.render_target(0)) {
						let device = self.device.as_ref().unwrap();
						let alloc = self.alloca.as_ref().unwrap();

						rendering.init_pipeline(
							device,
							alloc,
							target.color_format(),
							target.depth_format(),
						);

						if let Some(mat) = rendering.default_material() {
							let cube_mesh = Mesh::create_cube(device, alloc, 1.0);
							let cube_transform = TransformComponent::default();
							self.scene.spawn((
								cube_transform,
								MeshRendererComponent::new(cube_mesh, mat),
							));
						}
					}
					display.request_redraw();
					self.display = Some(display);
					mochi_disp!("Display and Swapchain attached successfully");
				}
				Err(err) => {
					mochi_error!("Failed to initialize display: {err}");
					event_loop.exit();
				}
			}
		}
	}

	fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
		match event {
			WindowEvent::CloseRequested => {
				mochi_log!("Window close requested, shutting down");
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
				mochi_log!("Escape pressed, shutting down");
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

				for (_e, (t, _r)) in self.scene.world_mut().query_mut::<(&mut TransformComponent, &MeshRendererComponent)>() {
					let rot_delta = mochi_math::Quat::from_axis_angle(
						Vec3::new(0.6, 1.0, 0.4).normalize(),
						mochi_math::Rad::new(1.0 * dt),
					);
					t.rotation = (t.rotation * rot_delta).normalize();
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

				if let (Some(display), Some(rendering), Some(sync), Some(cmdmng), Some(device), Some(alloc)) = (
					&mut self.display,
					&mut self.rendering_system,
					&mut self.syncer,
					&self.cmdmng,
					&self.device,
					&self.alloca,
				) {
					let time = self.start_time.elapsed().as_secs_f32();
					rendering.render(sync, cmdmng, display, device, alloc, &self.scene, time, dt);

					self.frame_count += 1;

					if let Ok(limit_str) = std::env::var("MOCHI_TEST_EXIT_FRAMES") {
						if let Ok(limit) = limit_str.parse::<u64>() {
							if self.frame_count >= limit {
								mochi_log!("Reached test frame limit ({limit}), exiting");
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
		mochi_log!("Cleaning up Engine resources");
		self.audio_system = None;

		if let Some(device) = &self.device {
			device.wait_idle();
		}

		self.scene = Scene::new();
		self.rendering_system = None;
		self.display = None;

		self.syncer = None;
		self.cmdmng = None;
		self.alloca = None;
		self.device = None;

		mochi_disp!("Cleanup completed cleanly");
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

	pub fn scene(&self) -> &Scene {
		self.app.scene()
	}

	pub fn scene_mut(&mut self) -> &mut Scene {
		self.app.scene_mut()
	}

	pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
		let event_loop = EventLoop::new()?;
		event_loop.set_control_flow(ControlFlow::Poll);
		event_loop.run_app(&mut self.app)?;
		Ok(())
	}
}
