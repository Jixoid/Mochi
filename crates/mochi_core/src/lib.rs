/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod config;

use std::time::Instant;
use mochi_math::Vec3;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;
use mochi_asset::SceneAsset;
use mochi_audio::AudioSystem;
use mochi_display::Display;
use mochi_rendering::RenderingSystem;
use mochi_rhi as rhi;
use mochi_scene::Scene;
use mochi_script::ScriptSystem;
use mochi_phi::PhysicsWorld;
use mochi_phi_tachyon::TachyonBackend;
use mochi_except::*;

pub use config::*;
pub use mochi_input::*;


pub struct EngineApp {
	title: String,
	width: u32,
	height: u32,
	start_time: Instant,
	last_frame_time: Option<Instant>,
	fps_timer: Instant,
	real_fps_counter: u32,
	display_fps_counter: u32,
	last_real_fps: u32,
	last_display_fps: u32,
	frame_count: u64,
	idle_cb: Option<Box<dyn FnMut(f32)>>,

	device: Option<rhi::Device>,
	alloca: Option<rhi::Allocator>,
	cmdmng: Option<rhi::CommandManager>,
	syncer: Option<rhi::Synchronizer>,

	rendering_system: Option<RenderingSystem>,
	audio_system: Option<AudioSystem>,
	script_system: ScriptSystem,
	physics_world: Option<PhysicsWorld<TachyonBackend>>,
	input: Input,
	display: Option<Display>,
	scene: Scene,
	config: ProjectConfig,
}

impl EngineApp {
	pub fn from_config(config: ProjectConfig) -> Self {
		// RHI
		let device =
			rhi::Device::new_with_options("MochiEngine", 0, 1, 0, 0, config.rendering.validation);
		let alloca = rhi::Allocator::new(&device);
		let cmdmng = rhi::CommandManager::new(&device);
		let syncer = rhi::Synchronizer::new(&device, RenderingSystem::MAX_FRAMES_IN_FLIGHT);

		// Audio
		let audio_system = AudioSystem::new("Mochi Audio", 0, 1, 0, 0);

		// SYS
		let rendering_system = RenderingSystem::new(&cmdmng).expect("failed to create RenderingSystem");
		let script_system = ScriptSystem::new();
		let input = Input::new();

		Self {
			title: config.window.title.clone(),
			width: config.window.width,
			height: config.window.height,
			start_time: Instant::now(),
			device: Some(device),
			alloca: Some(alloca),
			cmdmng: Some(cmdmng),
			syncer: Some(syncer),

			rendering_system: Some(rendering_system),
			audio_system: Some(audio_system),
			script_system,
			physics_world: None,
			input,
			display: None,
			scene: Scene::new(),
			config,
			last_frame_time: None,
			fps_timer: Instant::now(),
			real_fps_counter: 0,
			display_fps_counter: 0,
			last_real_fps: 0,
			last_display_fps: 0,
			frame_count: 0,
			idle_cb: None,
		}
	}

	pub fn new(title: &str, width: u32, height: u32) -> Self {
		let mut config = ProjectConfig::default();
		config.window.title = title.to_string();
		config.window.width = width;
		config.window.height = height;
		Self::from_config(config)
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

	pub fn config(&self) -> &ProjectConfig {
		&self.config
	}

	pub fn init_physics(&mut self) {
		let has_physics = self
			.scene
			.world()
			.query::<&mochi_scene::RigidBodyComponent>()
			.iter()
			.next()
			.is_some();

		if !has_physics {
			return;
		}

		let config = mochi_phi::PhysicsConfig {
			gravity: Vec3::new(0.0, -9.81, 0.0),
			fixed_timestep: 1.0 / 60.0,
			..Default::default()
		};

		match PhysicsWorld::<TachyonBackend>::new(config) {
			Ok(mut physics) => {
				let mut body_count = 0;
				for (_entity, (transform, rb)) in self.scene.world_mut().query_mut::<(
					&mochi_scene::TransformComponent,
					&mut mochi_scene::RigidBodyComponent,
				)>() {
					rb.def.position = transform.0.translation;
					rb.def.rotation = transform.0.rotation;
					if let Ok(handle) = physics.create_body(&rb.def) {
						rb.handle = handle;
						body_count += 1;
					}
				}

				for (_entity, (rb, col)) in self.scene.world_mut().query_mut::<(
					&mochi_scene::RigidBodyComponent,
					&mut mochi_scene::ColliderComponent,
				)>() {
					if rb.handle.is_valid()
						&& let Ok(col_handle) = physics.add_collider(rb.handle, &col.def)
					{
						col.handle = col_handle;
					}
				}

				let mut name_to_body = std::collections::HashMap::new();
				for (_entity, (name, rb)) in self
					.scene
					.world()
					.query::<(&mochi_scene::NameComponent, &mochi_scene::RigidBodyComponent)>()
					.iter()
				{
					if rb.handle.is_valid() {
						name_to_body.insert(name.0.clone(), rb.handle);
					}
				}

				let mut spring_count = 0;
				for (_entity, (rb, springs)) in self.scene.world_mut().query_mut::<(
					&mochi_scene::RigidBodyComponent,
					&mut mochi_scene::SpringJointsComponent,
				)>() {
					if !rb.handle.is_valid() {
						continue;
					}
					for joint in &mut springs.joints {
						if let Some(&target_body) = name_to_body.get(&joint.target_entity_name) {
							let def = mochi_phi::SpringJointDef {
								body_a: rb.handle,
								body_b: target_body,
								local_anchor_a: joint.local_anchor_a,
								local_anchor_b: joint.local_anchor_b,
								rest_length: joint.rest_length,
								stiffness: joint.stiffness,
								damping: joint.damping,
							};
							if let Ok(handle) = physics.add_spring_joint(&def) {
								joint.handle = handle;
								spring_count += 1;
							} else {
								mochi_warn!(
									"Failed to add spring joint between {:?} and '{}'",
									rb.handle,
									joint.target_entity_name
								);
							}
						} else {
							mochi_warn!(
								"Target entity '{}' for spring joint not found or has no rigid body",
								joint.target_entity_name
							);
						}
					}
				}

				if spring_count > 0 {
					mochi_disp!("Initialized physics with {body_count} rigid bodies and {spring_count} spring/suspension constraints");
				} else {
					mochi_disp!("Initialized physics with {body_count} rigid bodies");
				}
				self.physics_world = Some(physics);
			}
			Err(err) => {
				mochi_error!("Failed to initialize physics: {err}");
			}
		}
	}

	pub fn input(&self) -> &Input {
		&self.input
	}

	pub fn script_system(&self) -> &ScriptSystem {
		&self.script_system
	}

	pub fn script_system_mut(&mut self) -> &mut ScriptSystem {
		&mut self.script_system
	}
}

impl ApplicationHandler for EngineApp {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.display.is_none() {
			match Display::new(
				event_loop,
				self.device.as_ref().unwrap(),
				&self.title,
				self.width,
				self.height,
				self.config.window.vsync,
			) {
				Ok(mut display) => {
					if let (Some(rendering), Some(target)) =
						(&mut self.rendering_system, display.render_target(0))
					{
						let device = self.device.as_ref().unwrap();
						let alloc = self.alloca.as_ref().unwrap();

						rendering.init_pipeline(
							device,
							alloc,
							target.color_format(),
							target.depth_format(),
							display.msaa_samples(),
						);

						if self.config.rendering.frame_generation {
							rendering.set_frame_generation(device, alloc, &mut display, true);
							mochi_script::api::set_current_frame_gen(true);
						}

						let resolved_scene_path = self.config.resolve_path(&self.config.scene.start_scene);
						let scene_dir = resolved_scene_path.parent();
						let project_root = &self.config.root_dir;
						match SceneAsset::load_file(&resolved_scene_path) {
							Ok(scene_asset) => {
								mochi_disp!(
									"Loaded scene asset '{}' from {}",
									scene_asset.name,
									resolved_scene_path.display()
								);
								if let Err(err) = scene_asset.instantiate(
									device,
									alloc,
									rendering,
									&mut self.scene,
									scene_dir,
									Some(project_root),
								) {
									mochi_error!("Failed to instantiate scene: {err}");
								}
							}
							Err(err) => {
								mochi_error!(
									"Could not load scene '{}': {err}",
									resolved_scene_path.display()
								);
							}
						}
						self.init_physics();
					}
					display.request_redraw();
					self.input.set_cursor_captured(true);
					mochi_script::api::set_current_cursor_captured(true);
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

	fn device_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		_device_id: DeviceId,
		event: DeviceEvent,
	) {
		self.input.handle_device_event(&event);
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		_window_id: WindowId,
		event: WindowEvent,
	) {
		self.input.handle_window_event(&event);

		match event {
			WindowEvent::Focused(focused) => {
				if !focused && let Some(display) = &self.display {
					display.set_cursor_captured(false);
					self.input.set_cursor_captured(false);
					mochi_script::api::set_current_cursor_captured(false);
				}
			}
			WindowEvent::MouseInput {
				state: ElementState::Pressed,
				button: MouseButton::Left,
				..
			} => {
				if let Some(display) = &self.display
					&& !display.is_cursor_captured()
				{
					display.set_cursor_captured(true);
					self.input.set_cursor_captured(true);
					mochi_script::api::set_current_cursor_captured(true);
				}
			}
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
				if let Some(display) = &self.display
					&& display.is_cursor_captured()
				{
					mochi_log!("Escape pressed: releasing cursor capture");
					display.set_cursor_captured(false);
					self.input.set_cursor_captured(false);
					mochi_script::api::set_current_cursor_captured(false);
					return;
				}
				mochi_log!("Escape pressed, shutting down");
				event_loop.exit();
			}
			WindowEvent::KeyboardInput {
				event:
					KeyEvent {
						physical_key: PhysicalKey::Code(KeyCode::KeyR),
						state: ElementState::Pressed,
						..
					},
				..
			} => {
				if let Some(physics) = &mut self.physics_world {
					for (_entity, (transform, rb)) in self
						.scene
						.world_mut()
						.query_mut::<(
							&mut mochi_scene::TransformComponent,
							&mochi_scene::RigidBodyComponent,
						)>()
					{
						if rb.handle.is_valid() && rb.def.body_type == mochi_scene::BodyType::Dynamic {
							transform.0.translation = rb.def.position;
							transform.0.rotation = rb.def.rotation;
							let _ = physics.set_body_transform(rb.handle, rb.def.position, rb.def.rotation);
							let _ = physics.set_body_velocity(rb.handle, Vec3::ZERO, Vec3::ZERO);
						}
					}
					physics.clear_contacts();
					mochi_disp!("Reset physics bodies to initial positions and cleared contacts");
				}
			}
			WindowEvent::KeyboardInput {
				event:
					KeyEvent {
						physical_key: PhysicalKey::Code(KeyCode::Space) | PhysicalKey::Code(KeyCode::KeyJ),
						state: ElementState::Pressed,
						..
					},
				..
			} => {
				if let Some(physics) = &mut self.physics_world {
					for (_entity, rb) in self
						.scene
						.world()
						.query::<&mochi_scene::RigidBodyComponent>()
						.iter()
					{
						if rb.handle.is_valid() && rb.def.body_type == mochi_scene::BodyType::Dynamic {
							let _ = physics.apply_impulse(rb.handle, Vec3::new(0.0, 7.5, 0.0));
						}
					}
					mochi_disp!("Applied upward impulse to all dynamic cubes");
				}
			}
			WindowEvent::Resized(physical_size) => {
				if let Some(display) = &mut self.display {
					display.resize(physical_size.width, physical_size.height);
					display.request_redraw();
				}
			}
			WindowEvent::RedrawRequested => {
				let now = Instant::now();
				let dt = self
					.last_frame_time
					.map_or(0.016, |t| (now - t).as_secs_f32());
				self.last_frame_time = Some(now);
				let time = self.start_time.elapsed().as_secs_f32();

				let is_fg_present = self
					.rendering_system
					.as_ref()
					.is_some_and(|r| r.is_frame_gen_present_phase());

				if !is_fg_present {
					if let Some(cb) = &mut self.idle_cb {
						cb(dt);
					}

					if let Some(physics) = &mut self.physics_world {
						physics.step(dt);
						for (_entity, (transform, rb)) in self.scene.world_mut().query_mut::<(
							&mut mochi_scene::TransformComponent,
							&mochi_scene::RigidBodyComponent,
						)>() {
							if rb.handle.is_valid()
								&& let Ok((pos, rot)) = physics.body_transform(rb.handle)
							{
								transform.0.translation = pos;
								transform.0.rotation = rot;
							}
						}
					}

					// Execute QW scripts on active entities
					self
						.script_system
						.update(&mut self.scene, &self.input, dt, time);
				}

				if let Some(req_captured) = mochi_script::api::take_requested_cursor_capture()
					&& let Some(display) = &self.display
				{
					display.set_cursor_captured(req_captured);
					self.input.set_cursor_captured(req_captured);
					mochi_script::api::set_current_cursor_captured(req_captured);
				}

				let mut title_dirty = false;

				if let Some(req_aa) = mochi_script::api::take_requested_aa_mode() {
					if let (Some(display), Some(rendering), Some(device), Some(alloc)) = (
						&mut self.display,
						&mut self.rendering_system,
						&self.device,
						&self.alloca,
					) {
						rendering.set_aa_mode(device, alloc, display, req_aa);
						mochi_script::api::set_current_aa_mode(req_aa);
						mochi_script::api::set_current_msaa(req_aa.msaa_samples());
						title_dirty = true;
					}
				} else if let Some(req_msaa) = mochi_script::api::take_requested_msaa()
					&& let (Some(display), Some(rendering), Some(device), Some(alloc)) = (
						&mut self.display,
						&mut self.rendering_system,
						&self.device,
						&self.alloca,
					) {
					rendering.set_msaa(device, alloc, display, req_msaa);
					mochi_script::api::set_current_msaa(req_msaa);
					title_dirty = true;
				}

				if let Some(req_fg) = mochi_script::api::take_requested_frame_gen()
					&& let (Some(display), Some(rendering), Some(device), Some(alloc)) = (
						&mut self.display,
						&mut self.rendering_system,
						&self.device,
						&self.alloca,
					) {
					rendering.set_frame_generation(device, alloc, display, req_fg);
					mochi_script::api::set_current_frame_gen(req_fg);
					title_dirty = true;
				}

				if !is_fg_present {
					self.real_fps_counter += 1;
				}
				self.display_fps_counter += 1;

				if self.fps_timer.elapsed().as_secs() >= 1 {
					self.last_real_fps = self.real_fps_counter;
					self.last_display_fps = self.display_fps_counter;
					self.real_fps_counter = 0;
					self.display_fps_counter = 0;
					self.fps_timer = Instant::now();
					title_dirty = true;
				}

				if title_dirty
					&& let (Some(display), Some(rendering)) = (&self.display, &self.rendering_system)
				{
					let title = if rendering.frame_gen_enabled() {
						format!(
							"{} - [{}/{} FPS] Frame: {} (FG: ON) | AA: {}",
							self.title,
							self.last_real_fps,
							self.last_display_fps,
							self.frame_count,
							rendering.aa_mode().name()
						)
					} else {
						format!(
							"{} - [{} FPS] Frame: {} | AA: {}",
							self.title,
							self.last_real_fps,
							self.frame_count,
							rendering.aa_mode().name()
						)
					};
					display.window().set_title(&title);
				}

				if let Some(audio) = &self.audio_system {
					audio.update();
				}

				if let (
					Some(display),
					Some(rendering),
					Some(sync),
					Some(cmdmng),
					Some(device),
					Some(alloc),
				) = (
					&mut self.display,
					&mut self.rendering_system,
					&mut self.syncer,
					&self.cmdmng,
					&self.device,
					&self.alloca,
				) {
					rendering.render(sync, cmdmng, display, device, alloc, &self.scene, time, dt);

					if !is_fg_present {
						self.frame_count += 1;
					}

					if let Ok(limit_str) = std::env::var("MOCHI_TEST_EXIT_FRAMES")
						&& let Ok(limit) = limit_str.parse::<u64>()
						&& self.frame_count >= limit
					{
						mochi_log!("Reached test frame limit ({limit}), exiting");
						event_loop.exit();
						return;
					}

					display.request_redraw();
				}

				if !is_fg_present {
					self.input.reset_frame();
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
	pub fn from_config(config_path: &str) -> Self {
		let config = ProjectConfig::load_or_default(config_path);
		Self {
			app: EngineApp::from_config(config),
		}
	}

	pub fn with_config(config: ProjectConfig) -> Self {
		Self {
			app: EngineApp::from_config(config),
		}
	}

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

	pub fn config(&self) -> &ProjectConfig {
		self.app.config()
	}

	pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
		let event_loop = EventLoop::new()?;
		event_loop.set_control_flow(ControlFlow::Poll);
		event_loop.run_app(&mut self.app)?;
		Ok(())
	}
}
