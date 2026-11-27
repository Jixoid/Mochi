/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


#[cfg(unix)]
use std::os::unix::io::RawFd;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use mochi_asset::{DirectionalLightDesc, PointLightDesc, SceneAsset, SpotLightDesc, TransformDesc};
use mochi_math::{Quat, Rad, Transform, Vec3, Vec4};
use mochi_rendering::RenderingSystem;
use mochi_rhi::{Format as MochiFormat, ImageUsage, MsaaSamples, RenderTarget};
use mochi_rhi_ash::{
	Allocator as MochiAllocator, CommandManager as MochiCommandManager, Device as MochiDevice,
	ExportableImage, Image2 as MochiImage2, ImageView2 as MochiImageView2,
};
use mochi_scene::{
	CameraComponent, DirectionalLightComponent, Entity, MeshRendererComponent, PointLightComponent,
	Scene, SpotLightComponent, TransformComponent,
};
use nimbus::core::{ObjectID, TaskQueue, Window};
use nimbus::rhi_ash::{
	CommandManager as NimbusCommandManager, Device as NimbusDevice, Format as NimbusFormat,
	ImportedExternalTexture,
};
use nimbus::ui::Viewport;


use crate::scene_overlays::{ViewportOverlayOptions, populate_scene_overlays};

pub enum RenderCommand {
	Resize {
		width: u32,
		height: u32,
	},
	LoadScene {
		asset: SceneAsset,
		scene_dir: Option<PathBuf>,
		project_root: Option<PathBuf>,
	},
	UpdateCamera {
		yaw_deg: f32,
		pitch_deg: f32,
		distance: f32,
		target: [f32; 3],
	},
	UpdateTransform {
		index: usize,
		transform: TransformDesc,
	},
	UpdateDirectionalLight {
		index: usize,
		desc: DirectionalLightDesc,
	},
	UpdatePointLight {
		index: usize,
		desc: PointLightDesc,
	},
	UpdateSpotLight {
		index: usize,
		desc: SpotLightDesc,
	},
	UpdateMeshFlags {
		index: usize,
		is_visible: bool,
		cast_shadows: bool,
	},
	AddEntity {
		transform: TransformDesc,
	},
	RemoveEntity {
		index: usize,
	},
	SetSelection {
		index: Option<usize>,
	},
	SetOverlays(ViewportOverlayOptions),
	Deactivate,
	Shutdown,
}

enum RenderEvent {
	#[cfg(unix)]
	ExportedTarget {
		fd: RawFd,
		allocation_size: u64,
		width: u32,
		height: u32,
	},
	FrameReady,
}


/// Non-blocking handle used by the UI thread (`editor.rs`) to send scene, camera,
/// and incremental ECS delta commands to Mochi's dedicated background render thread.
#[derive(Clone)]
pub struct ViewportController {
	cmd_tx: Sender<RenderCommand>,
	viewport_node_id: Arc<Mutex<Option<ObjectID>>>,
	loading_node_id: Arc<Mutex<Option<ObjectID>>>,
}

impl ViewportController {
	#[cfg(test)]
	pub(crate) fn new_dummy() -> Self {
		let (cmd_tx, _cmd_rx) = mpsc::channel();
		Self {
			cmd_tx,
			viewport_node_id: Arc::new(Mutex::new(None)),
			loading_node_id: Arc::new(Mutex::new(None)),
		}
	}

	pub fn set_viewport_node(&self, id: Option<ObjectID>) {
		*self.viewport_node_id.lock().unwrap() = id;
	}

	pub fn set_loading_node(&self, id: Option<ObjectID>) {
		*self.loading_node_id.lock().unwrap() = id;
	}

	pub fn resize(&self, width: u32, height: u32) {
		if width >= 2 && height >= 2 {
			let _ = self.cmd_tx.send(RenderCommand::Resize { width, height });
		}
	}

	pub fn load_scene(
		&self,
		asset: SceneAsset,
		scene_dir: Option<PathBuf>,
		project_root: Option<PathBuf>,
	) {
		let _ = self.cmd_tx.send(RenderCommand::LoadScene {
			asset,
			scene_dir,
			project_root,
		});
	}

	pub fn update_camera(&self, yaw_deg: f32, pitch_deg: f32, distance: f32, target: [f32; 3]) {
		let _ = self.cmd_tx.send(RenderCommand::UpdateCamera {
			yaw_deg,
			pitch_deg,
			distance,
			target,
		});
	}

	pub fn update_transform(&self, index: usize, transform: TransformDesc) {
		let _ = self
			.cmd_tx
			.send(RenderCommand::UpdateTransform { index, transform });
	}

	pub fn update_directional_light(&self, index: usize, desc: DirectionalLightDesc) {
		let _ = self
			.cmd_tx
			.send(RenderCommand::UpdateDirectionalLight { index, desc });
	}

	pub fn update_point_light(&self, index: usize, desc: PointLightDesc) {
		let _ = self
			.cmd_tx
			.send(RenderCommand::UpdatePointLight { index, desc });
	}

	pub fn update_spot_light(&self, index: usize, desc: SpotLightDesc) {
		let _ = self
			.cmd_tx
			.send(RenderCommand::UpdateSpotLight { index, desc });
	}

	pub fn update_mesh_flags(&self, index: usize, is_visible: bool, cast_shadows: bool) {
		let _ = self.cmd_tx.send(RenderCommand::UpdateMeshFlags {
			index,
			is_visible,
			cast_shadows,
		});
	}

	pub fn add_entity(&self, transform: TransformDesc) {
		let _ = self.cmd_tx.send(RenderCommand::AddEntity { transform });
	}

	pub fn remove_entity(&self, index: usize) {
		let _ = self.cmd_tx.send(RenderCommand::RemoveEntity { index });
	}

	pub fn set_selection(&self, index: Option<usize>) {
		let _ = self.cmd_tx.send(RenderCommand::SetSelection { index });
	}

	pub fn set_overlays(&self, overlays: ViewportOverlayOptions) {
		let _ = self.cmd_tx.send(RenderCommand::SetOverlays(overlays));
	}

	pub fn deactivate(&self) {
		*self.viewport_node_id.lock().unwrap() = None;
		*self.loading_node_id.lock().unwrap() = None;
		let _ = self.cmd_tx.send(RenderCommand::Deactivate);
	}
}


// Fields are ordered so child GPU resources drop BEFORE `cmdmng`, `alloc`, and `device`.
struct MochiGpuContext {
	depth_view: Option<MochiImageView2>,
	depth_image: Option<MochiImage2>,
	export_image: Option<ExportableImage>,
	entity_map: Vec<Vec<Entity>>,
	scene: Scene,
	rendering: RenderingSystem,
	cmdmng: MochiCommandManager,
	alloc: MochiAllocator,
	device: MochiDevice,
}

impl Drop for MochiGpuContext {
	fn drop(&mut self) {
		self.device.wait_idle();
	}
}

impl MochiGpuContext {
	fn new() -> Result<Self, String> {
		let mut device = MochiDevice::new_with_options("Mochi Studio Viewport", 0, 1, 0, 0, true);
		let alloc = MochiAllocator::new(&device);
		device.init_descriptor_heap(&alloc);
		let cmdmng = MochiCommandManager::new(&device);

		let mut rendering = RenderingSystem::new(&cmdmng)?;
		rendering.init_pipeline(
			&device,
			&alloc,
			MochiFormat::V4Norm8U,
			MochiFormat::Depth32f,
			MsaaSamples::Off,
		);

		Ok(Self {
			depth_view: None,
			depth_image: None,
			export_image: None,
			entity_map: Vec::new(),
			scene: Scene::new(),
			rendering,
			cmdmng,
			alloc,
			device,
		})
	}

	#[cfg(unix)]
	fn recreate_export_target(&mut self, width: u32, height: u32) -> Result<(RawFd, u64), String> {
		self.device.wait_idle();
		self.export_image = None;
		self.depth_view = None;
		self.depth_image = None;

		let export_img = ExportableImage::new(self.device.raw(), width, height, MochiFormat::V4Norm8U)
			.map_err(|e| format!("{e}"))?;

		let depth_img = MochiImage2::builder(width, height, MochiFormat::Depth32f)
			.usage(ImageUsage::DepthStencilAttachment | ImageUsage::Sampled)
			.build(&self.alloc);
		let depth_view = self
			.device
			.create_image_view(&depth_img, MochiFormat::Depth32f)?;

		let fd = export_img
			.export_fd(self.device.raw())
			.map_err(|e| format!("{e}"))?;
		let alloc_size = export_img.allocation_size();

		self.export_image = Some(export_img);
		self.depth_image = Some(depth_img);
		self.depth_view = Some(depth_view);

		Ok((fd, alloc_size))
	}
}


/// UI-thread side of the external-memory bridge. Never blocks on 3D rendering or asset loading.
pub struct StudioViewportBridge {
	imported: Option<ImportedExternalTexture>,
	viewport_node_id: Arc<Mutex<Option<ObjectID>>>,
	loading_node_id: Arc<Mutex<Option<ObjectID>>>,
	event_rx: Receiver<RenderEvent>,
	cmd_tx: Sender<RenderCommand>,
	worker: Option<JoinHandle<()>>,
}

impl StudioViewportBridge {
	pub fn spawn(ui_tasks: TaskQueue) -> (Self, ViewportController) {
		let (cmd_tx, cmd_rx) = mpsc::channel::<RenderCommand>();
		let (event_tx, event_rx) = mpsc::channel::<RenderEvent>();
		let viewport_node_id = Arc::new(Mutex::new(None));
		let loading_node_id = Arc::new(Mutex::new(None));

		let worker = thread::Builder::new()
			.name("mochi-viewport-render".to_string())
			.spawn(move || {
				mochi_render_thread_main(cmd_rx, event_tx, ui_tasks);
			})
			.expect("Failed to spawn Mochi viewport render thread");

		let controller = ViewportController {
			cmd_tx: cmd_tx.clone(),
			viewport_node_id: Arc::clone(&viewport_node_id),
			loading_node_id: Arc::clone(&loading_node_id),
		};

		let bridge = Self {
			imported: None,
			viewport_node_id,
			loading_node_id,
			event_rx,
			cmd_tx,
			worker: Some(worker),
		};

		(bridge, controller)
	}

	/// Non-blocking poll called on Nimbus's `on_pre_render` hook.
	pub fn poll(
		&mut self,
		nimbus_device: &NimbusDevice,
		nimbus_cmdmng: &NimbusCommandManager,
		window: &mut Window,
	) {
		#[cfg(unix)]
		let mut latest_target: Option<(RawFd, u64, u32, u32)> = None;
		let mut frame_ready = false;

		while let Ok(ev) = self.event_rx.try_recv() {
			match ev {
				#[cfg(unix)]
				RenderEvent::ExportedTarget {
					fd,
					allocation_size,
					width,
					height,
				} => {
					// If multiple resizes arrived before UI polled, close superseded fd to prevent leaks
					if let Some((old_fd, _, _, _)) =
						latest_target.replace((fd, allocation_size, width, height))
					{
						close_raw_fd(old_fd);
					}
				}
				RenderEvent::FrameReady => {
					frame_ready = true;
				}
			}
		}

		let viewport_id = *self.viewport_node_id.lock().unwrap();

		#[cfg(unix)]
		if let Some((fd, alloc_size, width, height)) = latest_target {
			nimbus_device.wait_idle();
			let existing_tex_id = self.imported.as_ref().map(|t| t.texture_id());
			self.imported = None;

			match ImportedExternalTexture::import_opaque_fd(
				nimbus_device.raw(),
				fd,
				alloc_size,
				width,
				height,
				NimbusFormat::V4Norm8U,
				existing_tex_id,
			) {
				Ok(imported) => {
					if let Err(e) = imported.transition_to_shader_read(nimbus_cmdmng.raw()) {
						eprintln!("[Mochi Studio] Imported image transition failed: {e}");
					}
					let tex_id = imported.texture_id();
					self.imported = Some(imported);

					if let Some(vp_id) = viewport_id
						&& let Some(vp) = window.tree_mut().get_widget_mut::<Viewport>(vp_id)
					{
						vp.set_texture_id(Some(tex_id));
					}
					window.tree_mut().needs_redraw = true;
				}
				Err(e) => {
					eprintln!("[Mochi Studio] External memory import failed: {e}");
				}
			}
		}

		if frame_ready {
			if let (Some(vp_id), Some(imported)) = (viewport_id, &self.imported)
				&& let Some(vp) = window.tree_mut().get_widget_mut::<Viewport>(vp_id)
				&& vp.texture_id != Some(imported.texture_id())
			{
				vp.set_texture_id(Some(imported.texture_id()));
			}
			if let Some(loading_id) = *self.loading_node_id.lock().unwrap() {
				window.tree_mut().set_visible(loading_id, false);
			}
			window.tree_mut().needs_redraw = true;
		}
	}
}

impl Drop for StudioViewportBridge {
	fn drop(&mut self) {
		// Drop Nimbus's imported view/memory before shutting down Mochi's exported memory
		self.imported = None;
		let _ = self.cmd_tx.send(RenderCommand::Shutdown);
		if let Some(handle) = self.worker.take() {
			let _ = handle.join();
		}
	}
}

#[cfg(unix)]
fn close_raw_fd(fd: RawFd) {
	use std::os::unix::io::FromRawFd;
	let _ = unsafe { std::fs::File::from_raw_fd(fd) };
}


fn mochi_render_thread_main(
	cmd_rx: Receiver<RenderCommand>,
	event_tx: Sender<RenderEvent>,
	ui_tasks: TaskQueue,
) {
	let mut mochi: Option<MochiGpuContext> = None;
	let mut current_size: (u32, u32) = (0, 0);
	let mut yaw_deg: f32 = 25.0;
	let mut pitch_deg: f32 = 20.0;
	let mut distance: f32 = 10.0;
	let mut target: [f32; 3] = [0.0, 0.0, 0.0];
	let mut selected_index: Option<usize> = None;
	let mut overlays = ViewportOverlayOptions::default();
	let mut active = false;
	let start_time = Instant::now();
	let mut last_frame_time = start_time;

	// Block until first command arrives
	while let Ok(first_cmd) = cmd_rx.recv() {
		let mut batch = vec![first_cmd];
		while let Ok(more) = cmd_rx.try_recv() {
			batch.push(more);
		}

		let mut pending_resize: Option<(u32, u32)> = None;
		let mut pending_scene: Option<(SceneAsset, Option<PathBuf>, Option<PathBuf>)> = None;
		let mut delta_cmds: Vec<RenderCommand> = Vec::new();
		let mut needs_render = false;
		let mut shutdown = false;

		for cmd in batch {
			match cmd {
				RenderCommand::Shutdown => {
					shutdown = true;
					break;
				}
				RenderCommand::Deactivate => {
					active = false;
					needs_render = false;
				}
				RenderCommand::Resize { width, height } => {
					active = true;
					pending_resize = Some((width, height));
				}
				RenderCommand::LoadScene {
					asset,
					scene_dir,
					project_root,
				} => {
					active = true;
					pending_scene = Some((asset, scene_dir, project_root));
					delta_cmds.clear();
				}
				RenderCommand::UpdateCamera {
					yaw_deg: y,
					pitch_deg: p,
					distance: d,
					target: t,
				} => {
					yaw_deg = y;
					pitch_deg = p;
					distance = d;
					target = t;
					needs_render = true;
				}
				RenderCommand::SetSelection { index } => {
					selected_index = index;
					needs_render = true;
				}
				RenderCommand::SetOverlays(opts) => {
					overlays = opts;
					needs_render = true;
				}
				other => {
					delta_cmds.push(other);
				}
			}
		}

		if shutdown {
			break;
		}

		if !active {
			continue;
		}

		// Lazily initialize Mochi's separate Vulkan device on the background thread
		if mochi.is_none() {
			match MochiGpuContext::new() {
				Ok(ctx) => {
					mochi = Some(ctx);
				}
				Err(e) => {
					eprintln!("[Mochi Studio] Mochi GPU context başlatılamadı: {e}");
					continue;
				}
			}
		}

		let Some(ctx) = mochi.as_mut() else {
			continue;
		};

		// Handle resize & external FD export
		if let Some((w, h)) = pending_resize
			&& (w, h) != current_size
		{
			#[cfg(unix)]
			match ctx.recreate_export_target(w, h) {
				Ok((fd, allocation_size)) => {
					current_size = (w, h);
					let _ = event_tx.send(RenderEvent::ExportedTarget {
						fd,
						allocation_size,
						width: w,
						height: h,
					});
					ui_tasks.push_tree(|tree| {
						tree.needs_redraw = true;
					});
					needs_render = true;
				}
				Err(e) => {
					eprintln!("[Mochi Studio] ExportableImage oluşturulamadı: {e}");
					continue;
				}
			}
		}

		// Full scene load (only when opening a new .mscene.ron file)
		if let Some((asset, scene_dir, project_root)) = pending_scene {
			ctx.device.wait_idle();
			ctx.scene = Scene::new();
			ctx.entity_map.clear();
			match asset.instantiate_tracked(
				&ctx.device,
				&ctx.alloc,
				&mut ctx.rendering,
				&mut ctx.scene,
				scene_dir.as_deref(),
				project_root.as_deref(),
			) {
				Ok(map) => {
					ctx.entity_map = map;
					needs_render = true;
				}
				Err(e) => {
					eprintln!("[Mochi Studio] 3D sahne yüklenemedi: {e}");
				}
			}
		}

		// Incremental in-memory ECS delta updates (ZERO disk I/O, ZERO scene reload!)
		for delta in delta_cmds {
			match delta {
				RenderCommand::UpdateTransform { index, transform } => {
					if let Some(entities) = ctx.entity_map.get(index) {
						let t = transform.to_transform();
						for &entity in entities {
							if let Ok(mut tc) = ctx.scene.world_mut().get::<&mut TransformComponent>(entity) {
								tc.0 = t;
							}
						}
						needs_render = true;
					}
				}
				RenderCommand::UpdateDirectionalLight { index, desc } => {
					if let Some(entities) = ctx.entity_map.get(index) {
						let dir = Vec3::new(desc.direction[0], desc.direction[1], desc.direction[2]);
						let len_sq = dir.x * dir.x + dir.y * dir.y + dir.z * dir.z;
						let dir_norm = if len_sq > 1e-6 {
							dir.normalize()
						} else {
							Vec3::new(0.0, -1.0, 0.0)
						};
						for &entity in entities {
							if let Ok(mut dl) = ctx
								.scene
								.world_mut()
								.get::<&mut DirectionalLightComponent>(entity)
							{
								dl.color = Vec3::new(desc.color[0], desc.color[1], desc.color[2]);
								dl.intensity = desc.intensity;
								dl.direction = dir_norm;
								dl.cast_shadows = desc.cast_shadows;
							}
						}
						needs_render = true;
					}
				}
				RenderCommand::UpdatePointLight { index, desc } => {
					if let Some(entities) = ctx.entity_map.get(index) {
						for &entity in entities {
							if let Ok(mut pl) = ctx
								.scene
								.world_mut()
								.get::<&mut PointLightComponent>(entity)
							{
								pl.color = Vec3::new(desc.color[0], desc.color[1], desc.color[2]);
								pl.intensity = desc.intensity;
								pl.radius = desc.radius;
							}
						}
						needs_render = true;
					}
				}
				RenderCommand::UpdateSpotLight { index, desc } => {
					if let Some(entities) = ctx.entity_map.get(index) {
						for &entity in entities {
							if let Ok(mut sl) = ctx.scene.world_mut().get::<&mut SpotLightComponent>(entity) {
								sl.color = Vec3::new(desc.color[0], desc.color[1], desc.color[2]);
								sl.intensity = desc.intensity;
								sl.radius = desc.radius;
								sl.inner_cone_angle_deg = desc.inner_cone_angle_deg;
								sl.outer_cone_angle_deg = desc.outer_cone_angle_deg;
								sl.cast_shadows = desc.cast_shadows;
							}
						}
						needs_render = true;
					}
				}
				RenderCommand::UpdateMeshFlags {
					index,
					is_visible,
					cast_shadows,
				} => {
					if let Some(entities) = ctx.entity_map.get(index) {
						for &entity in entities {
							if let Ok(mut mr) = ctx
								.scene
								.world_mut()
								.get::<&mut MeshRendererComponent>(entity)
							{
								mr.is_visible = is_visible;
								mr.cast_shadows = cast_shadows;
							}
						}
						needs_render = true;
					}
				}
				RenderCommand::AddEntity { transform } => {
					let entity = ctx
						.scene
						.spawn((TransformComponent(transform.to_transform()),));
					ctx.entity_map.push(vec![entity]);
					needs_render = true;
				}
				RenderCommand::RemoveEntity { index } if index < ctx.entity_map.len() => {
					let removed = ctx.entity_map.remove(index);
					for entity in removed {
						let _ = ctx.scene.world_mut().despawn(entity);
					}
					needs_render = true;
				}
				_ => {}
			}
		}

		if !needs_render || current_size.0 < 2 || current_size.1 < 2 {
			continue;
		}

		let (Some(export_img), Some(depth_img), Some(depth_view)) =
			(&ctx.export_image, &ctx.depth_image, &ctx.depth_view)
		else {
			continue;
		};

		let now = Instant::now();
		let time = (now - start_time).as_secs_f32();
		let dt = (now - last_frame_time).as_secs_f32().clamp(0.001, 0.1);
		last_frame_time = now;

		let yaw = Rad {
			rad: yaw_deg.to_radians(),
		};
		let pitch = Rad {
			rad: -pitch_deg.to_radians(),
		};
		let rotation = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), yaw)
			* Quat::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), pitch);
		let position = Vec3::new(target[0], target[1], target[2])
			+ rotation * Vec3::new(0.0, 0.0, distance.max(0.5));
		let cam_transform = TransformComponent(Transform {
			translation: position,
			rotation,
			scale: Vec3::ONE,
		});
		let camera = CameraComponent {
			fov_deg: 60.0,
			near: 0.1,
			far: 1000.0,
			is_active: true,
			clear_color: Vec4::new(0.07, 0.08, 0.11, 1.0),
		};

		let ash_rt = export_img.as_render_target(
			depth_img.raw().handle(),
			depth_view.raw().handle(),
			MochiFormat::Depth32f,
		);
		let render_target = RenderTarget::from_raw(ash_rt);

		let selected_entities = selected_index
			.and_then(|i| ctx.entity_map.get(i))
			.map(|v| v.as_slice())
			.unwrap_or(&[]);
		populate_scene_overlays(
			&ctx.scene,
			selected_entities,
			overlays,
			ctx.rendering.debug_draw_mut(),
		);

		ctx.rendering.render_to_target(
			&ctx.cmdmng,
			&ctx.device,
			&ctx.alloc,
			&render_target,
			current_size.0,
			current_size.1,
			&ctx.scene,
			&cam_transform,
			&camera,
			time,
			dt,
		);

		let _ = event_tx.send(RenderEvent::FrameReady);
		ui_tasks.push_tree(|tree| {
			tree.needs_redraw = true;
		});
	}

	// Explicitly drop MochiGpuContext on the worker thread before exiting
	drop(mochi);
}
