/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use crate::material::{BlendMode, MaterialInstance, MaterialStorage, MaterialTemplate, PassKind, TextureManager};
use crate::rendering::aa::AntiAliasingMode;
use crate::rendering::cluster::ClusterManager;
use crate::rendering::frame::SceneFrameBuffer;
use crate::rendering::queue::RenderQueue;
use crate::rendering::shadow::{CASCADE_COUNT, SHADOW_MAP_SIZE, ShadowManager, ShadowPushConstants};
use crate::rhi;
use crate::scene::{CameraComponent, DirectionalLightComponent, Scene, TransformComponent};
use mochi_except::*;


static PBR_VERT_SPV: &[u8] = include_bytes!("../../res/shaders/pbr.vert.spv");
static PBR_FRAG_SPV: &[u8] = include_bytes!("../../res/shaders/pbr.frag.spv");
static FULLSCREEN_VERT_SPV: &[u8] = include_bytes!("../../res/shaders/fullscreen.vert.spv");
static FXAA_FRAG_SPV: &[u8] = include_bytes!("../../res/shaders/fxaa.frag.spv");


#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct ObjectPushConstants {
	pub model: [f32; 16],
	pub scene_addr: u64,
	pub vertex_addr: u64,
	pub material_addr: u64,
	pub material_id: u32,
	pub flags: u32,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default)]
pub struct FxaaPushConstants {
	pub texture_id: u32,
	pub texel_w: f32,
	pub texel_h: f32,
	pub _pad: f32,
}

pub struct OffscreenSceneTarget {
	pub color_image: rhi::Image2,
	pub color_view: rhi::ImageView2,
	pub depth_image: rhi::Image2,
	pub depth_view: rhi::ImageView2,
	pub descriptor_id: u32,
	pub width: u32,
	pub height: u32,
	pub color_format: rhi::Format,
	pub depth_format: rhi::Format,
}


pub struct RenderingSystem {
	cmd_buffers: Vec<rhi::Command>,
	cmdmng: rhi::CommandManager,
	frame_buffer: SceneFrameBuffer,
	material_storage: MaterialStorage,
	render_queue: RenderQueue,
	texture_manager: Option<TextureManager>,
	default_template: Option<Arc<MaterialTemplate>>,
	transparent_template: Option<Arc<MaterialTemplate>>,
	default_material: Option<Arc<MaterialInstance>>,
	transparent_material: Option<Arc<MaterialInstance>>,
	aa_mode: AntiAliasingMode,
	offscreen_target: Option<OffscreenSceneTarget>,
	fxaa_pipeline: Option<rhi::Pipeline>,
	fxaa_meta: Option<rhi::PipelineMeta>,
	fxaa_sampler: Option<rhi::Sampler2>,
	cluster_manager: Option<ClusterManager>,
	shadow_manager: Option<ShadowManager>,
	shadow_queue: RenderQueue,
}

impl RenderingSystem {
	pub const MAX_FRAMES_IN_FLIGHT: u32 = 2;

	pub fn new(cmdmng: &rhi::CommandManager) -> Result<Self, String> {
		let mut cmd_buffers = Vec::with_capacity(Self::MAX_FRAMES_IN_FLIGHT as usize);
		for _ in 0..Self::MAX_FRAMES_IN_FLIGHT {
			cmd_buffers.push(cmdmng.allocate_graphic_command()?);
		}

		Ok(Self {
			cmd_buffers,
			cmdmng: cmdmng.clone(),
			frame_buffer: SceneFrameBuffer::new(),
			material_storage: MaterialStorage::new(),
			render_queue: RenderQueue::new(),
			texture_manager: None,
			default_template: None,
			transparent_template: None,
			default_material: None,
			transparent_material: None,
			aa_mode: AntiAliasingMode::Off,
			offscreen_target: None,
			fxaa_pipeline: None,
			fxaa_meta: None,
			fxaa_sampler: None,
			cluster_manager: None,
			shadow_manager: None,
			shadow_queue: RenderQueue::new(),
		})
	}

	pub fn init_pipeline(
		&mut self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		color_format: rhi::Format,
		depth_format: rhi::Format,
		samples: rhi::MsaaSamples,
	) {
		mochi_verb!("Initializing modern PBR graphics pipeline");

		let texture_manager = TextureManager::new(device, alloc, &self.cmdmng);
		self.texture_manager = Some(texture_manager);

		let template = MaterialTemplate::new_forward(
			device,
			PBR_VERT_SPV,
			PBR_FRAG_SPV,
			BlendMode::Opaque,
			color_format,
			depth_format,
			std::mem::size_of::<ObjectPushConstants>() as u32,
			samples,
		);

		let transparent_template = MaterialTemplate::new_forward(
			device,
			PBR_VERT_SPV,
			PBR_FRAG_SPV,
			BlendMode::Transparent,
			color_format,
			depth_format,
			std::mem::size_of::<ObjectPushConstants>() as u32,
			samples,
		);

		let default_mat = MaterialInstance::new_default(Arc::clone(&template));
		let transparent_mat = MaterialInstance::new_default(Arc::clone(&transparent_template));
		self.material_storage.register(Arc::clone(&default_mat));
		self.material_storage.register(Arc::clone(&transparent_mat));
		self.material_storage.update_gpu(device, alloc);

		let fxaa_vert = rhi::Shader::new(
			device,
			rhi::ShaderStage::Vertex,
			FULLSCREEN_VERT_SPV,
			"main",
		);
		let fxaa_frag = rhi::Shader::new(
			device,
			rhi::ShaderStage::Pixel,
			FXAA_FRAG_SPV,
			"main",
		);
		let fxaa_meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Pixel,
			std::mem::size_of::<FxaaPushConstants>() as u32,
		);
		let fxaa_pipeline = rhi::Pipeline::new_graphics(
			device,
			&fxaa_meta,
			&[&fxaa_vert, &fxaa_frag],
			rhi::PolygonMode::Fill,
			rhi::PrimitiveTopology::TriangleList,
			color_format,
			rhi::Format::Undefined,
			false,
			false,
			rhi::MsaaSamples::Off,
		);
		let fxaa_sampler = device
			.create_sampler(
				rhi::SamplerFilter::Linear,
				rhi::SamplerAddressMode::ClampToEdge,
			)
			.expect("Failed to create FXAA linear sampler");

		self.fxaa_meta = Some(fxaa_meta);
		self.fxaa_pipeline = Some(fxaa_pipeline);
		self.fxaa_sampler = Some(fxaa_sampler);

		match ClusterManager::new(device, alloc) {
			Ok(cm) => {
				mochi_disp!("Clustered Forward light manager initialized");
				self.cluster_manager = Some(cm);
			}
			Err(err) => {
				mochi_warn!("Failed to initialize Clustered Forward manager: {err}");
			}
		}

		match ShadowManager::new(device, alloc) {
			Ok(sm) => {
				mochi_disp!("Cascaded Shadow Map manager initialized (4 cascades, 2048x2048)");
				self.shadow_manager = Some(sm);
			}
			Err(err) => {
				mochi_warn!("Failed to initialize Cascaded Shadow Map manager: {err}");
			}
		}

		mochi_disp!(
			"PBR graphics pipeline initialized. Opaque Template ID: {}, Transparent Template ID: {}, Samples: {:?}",
			template.id(),
			transparent_template.id(),
			samples
		);

		self.default_template = Some(template);
		self.transparent_template = Some(transparent_template);
		self.default_material = Some(default_mat);
		self.transparent_material = Some(transparent_mat);
	}

	pub fn aa_mode(&self) -> AntiAliasingMode {
		self.aa_mode
	}

	pub fn set_aa_mode(
		&mut self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		display: &mut crate::display::Display,
		mode: AntiAliasingMode,
	) {
		if self.aa_mode == mode {
			return;
		}

		self.aa_mode = mode;

		let msaa_samples = mode.msaa_samples();
		self.set_msaa(device, display, msaa_samples);

		if mode.is_fxaa() {
			let width = display.width();
			let height = display.height();
			if let Some(target) = display.render_target(0) {
				self.ensure_offscreen_target(
					device,
					alloc,
					width,
					height,
					target.color_format(),
					target.depth_format(),
				);
			}
		}

		mochi_disp!("Anti-Aliasing mode set to: {}", mode.name());
	}

	pub fn ensure_offscreen_target(
		&mut self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		width: u32,
		height: u32,
		color_format: rhi::Format,
		depth_format: rhi::Format,
	) {
		if let Some(target) = &self.offscreen_target {
			if target.width == width && target.height == height {
				return;
			}
		}

		let ext = mochi_math::Extent::new([width, height]);
		let color_image = rhi::Image2::new(
			alloc,
			ext,
			color_format,
			rhi::ImageUsage::ColorAttachment | rhi::ImageUsage::Sampled,
			rhi::ImageTiling::Optimal,
			rhi::AllocationCreate::Dedicated,
			rhi::AllocationLocation::PreferDevice,
		);
		let color_view = device
			.create_image_view(&color_image, color_format)
			.expect("Failed to create offscreen color view");

		let depth_image = rhi::Image2::new(
			alloc,
			ext,
			depth_format,
			rhi::ImageUsage::DepthStencilAttachment,
			rhi::ImageTiling::Optimal,
			rhi::AllocationCreate::Dedicated,
			rhi::AllocationLocation::PreferDevice,
		);
		let depth_view = device
			.create_image_view(&depth_image, depth_format)
			.expect("Failed to create offscreen depth view");

		let descriptor_id = if let Some(sampler) = &self.fxaa_sampler {
			device.write_texture_descriptor(&color_view, sampler)
		} else {
			0
		};

		self.offscreen_target = Some(OffscreenSceneTarget {
			color_image,
			color_view,
			depth_image,
			depth_view,
			descriptor_id,
			width,
			height,
			color_format,
			depth_format,
		});
	}

	pub fn set_msaa(
		&mut self,
		device: &rhi::Device,
		display: &mut crate::display::Display,
		samples: rhi::MsaaSamples,
	) {
		if display.msaa_samples() == samples {
			return;
		}

		display.set_msaa(samples);

		if let Some(template) = &self.default_template {
			template.set_msaa(device, samples);
		}
		if let Some(transparent_template) = &self.transparent_template {
			transparent_template.set_msaa(device, samples);
		}

		mochi_disp!("MSAA changed to {:?}", samples);
	}

	pub fn cmdmng(&self) -> &rhi::CommandManager {
		&self.cmdmng
	}

	pub fn texture_manager(&self) -> Option<&TextureManager> {
		self.texture_manager.as_ref()
	}

	pub fn texture_manager_mut(&mut self) -> Option<&mut TextureManager> {
		self.texture_manager.as_mut()
	}

	pub fn default_material(&self) -> Option<Arc<MaterialInstance>> {
		self.default_material.as_ref().map(Arc::clone)
	}

	pub fn default_template(&self) -> Option<Arc<MaterialTemplate>> {
		self.default_template.as_ref().map(Arc::clone)
	}

	pub fn transparent_template(&self) -> Option<Arc<MaterialTemplate>> {
		self.transparent_template.as_ref().map(Arc::clone)
	}

	pub fn transparent_material(&self) -> Option<Arc<MaterialInstance>> {
		self.transparent_material.as_ref().map(Arc::clone)
	}

	pub fn material_storage_mut(&mut self) -> &mut MaterialStorage {
		&mut self.material_storage
	}

	pub fn render(
		&mut self,
		sync: &mut rhi::Synchronizer,
		cmdmng: &rhi::CommandManager,
		display: &mut crate::display::Display,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		scene: &Scene,
		time: f32,
		delta_time: f32,
	) {
		sync.begin_frame();
		let frame = sync.current_frame() as usize;
		let wait_sem = sync.image_available_semaphore();
		let img_idx = display.acquire_next_image_with_semaphore(&wait_sem);

		let width = display.width();
		let height = display.height();
		let aspect = if height > 0 { width as f32 / height as f32 } else { 1.0 };

		let (cam_transform, camera) = scene
			.active_camera()
			.map(|(_e, t, c)| (t, c))
			.unwrap_or_else(|| {
				let mut default_t = TransformComponent::default();
				default_t.translation = mochi_math::Vec3::new(0.0, 1.0, 3.0);
				(default_t, CameraComponent::default())
			});

		let clear_color = camera.clear_color;

		let mut sun_dir = mochi_math::Vec3::new(0.5, -1.0, 0.3).normalize();
		for (_e, (t, light)) in scene.world().query::<(&TransformComponent, &DirectionalLightComponent)>().iter() {
			sun_dir = t.rotation * light.direction;
			break;
		}

		let (cascade_view_proj, cascade_split_depths, cascade_shadow_ids) = if let Some(sm) = &mut self.shadow_manager {
			sm.update_cascades(&camera, &cam_transform, aspect, sun_dir);
			(sm.cascade_view_proj(), sm.cascade_split_depths(), sm.cascade_shadow_ids())
		} else {
			(
				[[[0.0; 16]; 4]; 1][0],
				[2.5, 8.0, 24.0, 80.0],
				[0; 4],
			)
		};

		let (grid_addr, pt_index_addr, spot_index_addr) = if let Some(cm) = &self.cluster_manager {
			(cm.grid_address(), cm.point_index_list_address(), cm.spot_index_list_address())
		} else {
			(0, 0, 0)
		};

		// Update frame scene buffer (Camera + Dynamic Lights + Shadows)
		self.frame_buffer.update(
			device,
			alloc,
			scene,
			&cam_transform,
			&camera,
			aspect,
			time,
			delta_time,
			grid_addr,
			pt_index_addr,
			spot_index_addr,
			width,
			height,
			cascade_view_proj,
			cascade_split_depths,
			cascade_shadow_ids,
		);

		// Extract packets for ShadowCaster pass
		self.shadow_queue.extract_from_scene(
			scene,
			PassKind::ShadowCaster,
			cam_transform.translation,
			&mut self.material_storage,
		);

		// Extract packets and sort for ForwardLit pass
		self.render_queue.extract_from_scene(
			scene,
			PassKind::ForwardLit,
			cam_transform.translation,
			&mut self.material_storage,
		);

		// Update material storage GPU buffer
		self.material_storage.update_gpu(device, alloc);

		let is_fxaa = self.aa_mode.is_fxaa();
		if is_fxaa {
			if let Some(target) = display.render_target(0) {
				self.ensure_offscreen_target(
					device,
					alloc,
					width,
					height,
					target.color_format(),
					target.depth_format(),
				);
			}
		}

		let scene_addr = self.frame_buffer.address();
		let material_addr = self.material_storage.buffer_address();

		let cmd = &mut self.cmd_buffers[frame];
		cmd.begin();

		// Cascaded Shadow Map render passes (4 cascades)
		if let Some(sm) = &self.shadow_manager {
			for c in 0..CASCADE_COUNT {
				let cascade = &sm.cascades[c];
				let cascade_target = device.create_depth_render_target(
					&cascade.image,
					&cascade.view,
					rhi::Format::Depth32f,
					SHADOW_MAP_SIZE,
					SHADOW_MAP_SIZE,
				);

				cmd.render_pass(&cascade_target, clear_color, |pass| {
					pass.set_viewport(0.0, 0.0, SHADOW_MAP_SIZE as f32, SHADOW_MAP_SIZE as f32, 0.0, 1.0);
					pass.set_scissor(0, 0, SHADOW_MAP_SIZE, SHADOW_MAP_SIZE);
					pass.bind_pipeline(&sm.pipeline);

					let view_proj = mochi_math::Mat4x4::from_array(cascade.view_proj);

					for packet in &self.shadow_queue.packets {
						let model = mochi_math::Mat4x4::from_array(packet.model_matrix);
						let mvp = (view_proj * model).to_array();

						let pc = ShadowPushConstants {
							mvp,
							vertex_addr: packet.vertex_buffer_addr,
						};

						pass.push_constant(
							rhi::ShaderStage::Vertex,
							0,
							&pc,
						);

						if packet.is_indexed && packet.index_count > 0 {
							if let Some(index_buf) = packet.mesh.index_buffer() {
								pass.bind_index_buffer(index_buf, 0, rhi::IndexType::Uint32);
								pass.draw_indexed(packet.index_count, 1, 0, 0, 0);
							} else {
								pass.draw(packet.vertex_count, 1, 0, 0);
							}
						} else {
							pass.draw(packet.vertex_count, 1, 0, 0);
						}
					}
				});
			}
		}

		// Clustered Forward compute dispatch (Camera AABBs + Light Culling)
		if let Some(cm) = &mut self.cluster_manager {
			let proj = camera.projection_matrix(aspect);
			let inv_proj = proj.inverse();
			cm.update_camera(cmd, inv_proj.to_array(), camera.near, camera.far);
			cm.cull_lights(cmd, scene_addr);
		}

		if is_fxaa {
			if let Some(offscreen) = &self.offscreen_target {
				let scene_target = device.create_render_target(
					&offscreen.color_image,
					&offscreen.color_view,
					offscreen.color_format,
					&offscreen.depth_image,
					&offscreen.depth_view,
					offscreen.depth_format,
					width,
					height,
				);

				cmd.render_pass(&scene_target, clear_color, |pass| {
					Self::record_scene_packets(
						pass,
						width,
						height,
						&self.render_queue,
						scene_addr,
						material_addr,
					);
				});

				if let (Some(swapchain_target), Some(fxaa_pipeline)) = (
					display.render_target(img_idx),
					&self.fxaa_pipeline,
				) {
					let target_no_depth = swapchain_target.without_depth();
					cmd.render_pass(&target_no_depth, clear_color, |pass| {
						pass.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
						pass.set_scissor(0, 0, width, height);
						pass.bind_pipeline(fxaa_pipeline);

						let pc = FxaaPushConstants {
							texture_id: offscreen.descriptor_id,
							texel_w: 1.0 / width as f32,
							texel_h: 1.0 / height as f32,
							_pad: 0.0,
						};

						pass.push_constant(
							rhi::ShaderStage::Pixel,
							0,
							&pc,
						);

						pass.draw(3, 1, 0, 0);
					});
				}
			}
		} else if let Some(target) = display.render_target(img_idx) {
			cmd.render_pass(&target, clear_color, |pass| {
				Self::record_scene_packets(
					pass,
					width,
					height,
					&self.render_queue,
					scene_addr,
					material_addr,
				);
			});
		}

		cmd.end();

		let sig_sem = display.render_finished_semaphore(img_idx);
		let fence = sync.in_flight_fence();
		cmdmng.submit_with_sync(cmd, Some(&wait_sem), Some(&sig_sem), Some(&fence));
		sync.end_frame();

		display.present_with_semaphore(img_idx, &sig_sem);
	}

	fn record_scene_packets(
		pass: &mut rhi::Command,
		width: u32,
		height: u32,
		render_queue: &RenderQueue,
		scene_addr: u64,
		material_addr: u64,
	) {
		pass.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
		pass.set_scissor(0, 0, width, height);

		let mut currently_bound_pipeline_id: Option<u16> = None;

		for packet in &render_queue.packets {
			let template = packet.material.template();
			let template_id = template.id();

			if currently_bound_pipeline_id != Some(template_id) {
				if let Some(pipeline) = template.pipeline(PassKind::ForwardLit) {
					pass.bind_pipeline(&pipeline);
					currently_bound_pipeline_id = Some(template_id);
				}
			}

			let pc = ObjectPushConstants {
				model: packet.model_matrix,
				scene_addr,
				vertex_addr: packet.vertex_buffer_addr,
				material_addr,
				material_id: packet.material_id,
				flags: 0,
			};

			pass.push_constant(
				rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel,
				0,
				&pc,
			);

			if packet.is_indexed && packet.index_count > 0 {
				if let Some(index_buf) = packet.mesh.index_buffer() {
					pass.bind_index_buffer(index_buf, 0, rhi::IndexType::Uint32);
					pass.draw_indexed(packet.index_count, 1, 0, 0, 0);
				} else {
					pass.draw(packet.vertex_count, 1, 0, 0);
				}
			} else {
				pass.draw(packet.vertex_count, 1, 0, 0);
			}
		}
	}
}
