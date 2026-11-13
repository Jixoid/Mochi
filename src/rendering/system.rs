/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use crate::material::{
	BlendMode, MaterialInstance, MaterialStorage, MaterialTemplate, PassKind, TextureManager,
};
use crate::rendering::aa::AntiAliasingMode;
use crate::rendering::cluster::ClusterManager;
use crate::rendering::frame::{MAX_DIRECTIONAL_LIGHTS, SceneFrameBuffer};
use crate::rendering::queue::RenderQueue;
use crate::rendering::shadow::{CASCADE_COUNT, SHADOW_MAP_SIZE, ShadowManager, ShadowPushConstants};
use crate::rendering::smaa::{
	SmaaBlendPushConstants, SmaaEdgePushConstants, SmaaManager, SmaaNeighborPushConstants,
	SmaaResolvePushConstants,
};
use crate::rendering::velocity::{VelocityManager, VelocityPushConstants};
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

pub struct MsaaOffscreenAttachments {
	pub color_image: rhi::Image2,
	pub color_view: rhi::ImageView2,
	pub depth_image: rhi::Image2,
	pub depth_view: rhi::ImageView2,
}

pub struct OffscreenSceneTarget {
	pub color_image: rhi::Image2,
	pub color_view: rhi::ImageView2,
	pub depth_image: rhi::Image2,
	pub depth_view: rhi::ImageView2,
	pub descriptor_id: u32,
	pub depth_descriptor_id: u32,
	pub width: u32,
	pub height: u32,
	pub color_format: rhi::Format,
	pub depth_format: rhi::Format,
	pub msaa_samples: rhi::MsaaSamples,
	pub msaa_attachments: Option<MsaaOffscreenAttachments>,
}

pub struct HistoryTarget {
	pub image: rhi::Image2,
	pub view: rhi::ImageView2,
	pub descriptor_id: u32,
	pub width: u32,
	pub height: u32,
	pub format: rhi::Format,
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
	history_target: Option<HistoryTarget>,
	fxaa_pipeline: Option<rhi::Pipeline>,
	fxaa_meta: Option<rhi::PipelineMeta>,
	fxaa_sampler: Option<rhi::Sampler2>,
	point_sampler: Option<rhi::Sampler2>,
	smaa_manager: Option<SmaaManager>,
	velocity_manager: Option<VelocityManager>,
	cluster_manager: Option<ClusterManager>,
	shadow_manager: Option<ShadowManager>,
	shadow_queue: RenderQueue,
	frame_index: u64,
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
			history_target: None,
			fxaa_pipeline: None,
			fxaa_meta: None,
			fxaa_sampler: None,
			point_sampler: None,
			smaa_manager: None,
			velocity_manager: None,
			cluster_manager: None,
			shadow_manager: None,
			shadow_queue: RenderQueue::new(),
			frame_index: 0,
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
		let fxaa_frag = rhi::Shader::new(device, rhi::ShaderStage::Pixel, FXAA_FRAG_SPV, "main");
		let fxaa_meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Pixel,
			std::mem::size_of::<FxaaPushConstants>() as u32,
		);
		let fxaa_pipeline = rhi::GraphicsPipelineBuilder::new(device, &fxaa_meta)
			.shaders(&[&fxaa_vert, &fxaa_frag])
			.color_format(color_format)
			.depth_write(false)
			.build();
		let fxaa_sampler = device
			.create_sampler(
				rhi::SamplerFilter::Linear,
				rhi::SamplerAddressMode::ClampToEdge,
			)
			.expect("Failed to create FXAA linear sampler");

		self.fxaa_meta = Some(fxaa_meta);
		self.fxaa_pipeline = Some(fxaa_pipeline);
		self.fxaa_sampler = Some(fxaa_sampler);

		let smaa_manager = SmaaManager::new(device, alloc, &self.cmdmng, color_format);
		self.smaa_manager = Some(smaa_manager);

		let point_sampler = device
			.create_sampler(
				rhi::SamplerFilter::Nearest,
				rhi::SamplerAddressMode::ClampToEdge,
			)
			.expect("Failed to create point clamp sampler");
		self.point_sampler = Some(point_sampler);

		let velocity_manager = VelocityManager::new(device);
		self.velocity_manager = Some(velocity_manager);

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

	pub fn velocity_target(&self) -> Option<&crate::rendering::velocity::VelocityTarget> {
		self
			.velocity_manager
			.as_ref()
			.and_then(|v| v.target.as_ref())
	}

	pub fn history_target(&self) -> Option<&HistoryTarget> {
		self.history_target.as_ref()
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

		let msaa_samples = mode.required_msaa();
		let display_msaa = if mode.is_post_process() {
			rhi::MsaaSamples::Off
		} else {
			msaa_samples
		};

		if display.msaa_samples() != display_msaa {
			display.set_msaa(display_msaa);
		}

		if let Some(template) = &self.default_template {
			template.set_msaa(device, msaa_samples);
		}
		if let Some(transparent_template) = &self.transparent_template {
			transparent_template.set_msaa(device, msaa_samples);
		}

		if mode.is_post_process() {
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
					msaa_samples,
				);
			}
		}

		if let Some(smaa) = &mut self.smaa_manager {
			smaa.has_history = false;
		}

		mochi_disp!("Anti-Aliasing mode set to: {}", mode.name());
	}

	#[allow(clippy::too_many_arguments)]
	pub fn ensure_offscreen_target(
		&mut self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		width: u32,
		height: u32,
		color_format: rhi::Format,
		depth_format: rhi::Format,
		msaa_samples: rhi::MsaaSamples,
	) {
		let needs_msaa = msaa_samples.count() > 1;

		if let Some(target) = &mut self.offscreen_target
			&& target.width == width
			&& target.height == height
			&& target.color_format == color_format
			&& target.depth_format == depth_format
		{
			if target.msaa_samples == msaa_samples {
				return;
			}

			target.msaa_samples = msaa_samples;
			if needs_msaa {
				let (msaa_col_img, msaa_col_view, msaa_depth_img, msaa_depth_view) =
					Self::create_msaa_attachments(
						device,
						alloc,
						width,
						height,
						color_format,
						depth_format,
						msaa_samples,
					);
				target.msaa_attachments = Some(MsaaOffscreenAttachments {
					color_image: msaa_col_img,
					color_view: msaa_col_view,
					depth_image: msaa_depth_img,
					depth_view: msaa_depth_view,
				});
			} else {
				target.msaa_attachments = None;
			}
			return;
		}

		let color_image = rhi::Image2::builder(width, height, color_format)
			.usage(rhi::ImageUsage::ColorAttachment | rhi::ImageUsage::Sampled)
			.build(alloc);
		let color_view = device
			.create_image_view(&color_image, color_format)
			.expect("Failed to create offscreen color view");

		let depth_image = rhi::Image2::builder(width, height, depth_format)
			.usage(rhi::ImageUsage::DepthStencilAttachment | rhi::ImageUsage::Sampled)
			.build(alloc);
		let depth_view = device
			.create_image_view(&depth_image, depth_format)
			.expect("Failed to create offscreen depth view");

		let descriptor_id = if let Some(sampler) = &self.fxaa_sampler {
			device.write_texture_descriptor(&color_view, sampler)
		} else {
			0
		};

		let depth_descriptor_id = if let Some(sampler) = &self.point_sampler {
			device.write_texture_descriptor(&depth_view, sampler)
		} else {
			0
		};

		let msaa_attachments = if needs_msaa {
			let (msaa_col_img, msaa_col_view, msaa_depth_img, msaa_depth_view) =
				Self::create_msaa_attachments(
					device,
					alloc,
					width,
					height,
					color_format,
					depth_format,
					msaa_samples,
				);
			Some(MsaaOffscreenAttachments {
				color_image: msaa_col_img,
				color_view: msaa_col_view,
				depth_image: msaa_depth_img,
				depth_view: msaa_depth_view,
			})
		} else {
			None
		};

		self.offscreen_target = Some(OffscreenSceneTarget {
			color_image,
			color_view,
			depth_image,
			depth_view,
			descriptor_id,
			depth_descriptor_id,
			width,
			height,
			color_format,
			depth_format,
			msaa_samples,
			msaa_attachments,
		});
	}

	fn create_msaa_attachments(
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		width: u32,
		height: u32,
		color_format: rhi::Format,
		depth_format: rhi::Format,
		samples: rhi::MsaaSamples,
	) -> (rhi::Image2, rhi::ImageView2, rhi::Image2, rhi::ImageView2) {
		let col_img = rhi::Image2::builder(width, height, color_format)
			.usage(rhi::ImageUsage::ColorAttachment | rhi::ImageUsage::TransientAttachment)
			.samples(samples)
			.build(alloc);
		let col_view = device
			.create_image_view(&col_img, color_format)
			.expect("Failed to create MSAA offscreen color view");

		let depth_img = rhi::Image2::builder(width, height, depth_format)
			.usage(rhi::ImageUsage::DepthStencilAttachment | rhi::ImageUsage::TransientAttachment)
			.samples(samples)
			.build(alloc);
		let depth_view = device
			.create_image_view(&depth_img, depth_format)
			.expect("Failed to create MSAA offscreen depth view");

		(col_img, col_view, depth_img, depth_view)
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

	#[allow(clippy::too_many_arguments)]
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
		let aspect = if height > 0 {
			width as f32 / height as f32
		} else {
			1.0
		};

		let (cam_transform, camera) = scene
			.active_camera()
			.map(|(_e, t, c)| (t, c))
			.unwrap_or_else(|| {
				let default_t = TransformComponent(mochi_math::Transform::from_translation(
					mochi_math::Vec3::new(0.0, 1.0, 3.0),
				));
				(default_t, CameraComponent::default())
			});

		let clear_color = camera.clear_color;

		let mut cascade_view_proj =
			[[[mochi_math::Mat4x4::IDENTITY.to_array(); 4]; MAX_DIRECTIONAL_LIGHTS]; 1][0];
		let mut cascade_shadow_ids = [[0u32; 4]; MAX_DIRECTIONAL_LIGHTS];
		let cascade_split_depths = [2.5, 8.0, 24.0, 80.0];

		if let Some(sm) = &mut self.shadow_manager {
			for (light_idx, (_e, (t, light))) in scene
				.world()
				.query::<(&TransformComponent, &DirectionalLightComponent)>()
				.iter()
				.take(MAX_DIRECTIONAL_LIGHTS)
				.enumerate()
			{
				if light.cast_shadows {
					let dir = t.rotation * light.direction;
					sm.ensure_cascade_set(device, alloc, light_idx).ok();
					sm.update_cascades(light_idx, &camera, &cam_transform, aspect, dir);
					cascade_view_proj[light_idx] = sm.cascade_view_proj(light_idx);
					cascade_shadow_ids[light_idx] = sm.cascade_shadow_ids(light_idx);
				}
			}
		}

		let (grid_addr, pt_index_addr, spot_index_addr) = if let Some(cm) = &self.cluster_manager {
			(
				cm.grid_address(),
				cm.point_index_list_address(),
				cm.spot_index_list_address(),
			)
		} else {
			(0, 0, 0)
		};

		self.frame_index += 1;
		let jitter = if self.aa_mode.needs_camera_jitter() {
			crate::rendering::aa::get_subpixel_jitter(self.aa_mode, self.frame_index)
		} else {
			(0.0, 0.0)
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
			jitter,
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

		let is_post_process = self.aa_mode.is_post_process();
		if is_post_process && let Some(target) = display.render_target(0) {
			let msaa_samples = self.aa_mode.required_msaa();
			self.ensure_offscreen_target(
				device,
				alloc,
				width,
				height,
				target.color_format(),
				target.depth_format(),
				msaa_samples,
			);
		}

		let scene_addr = self.frame_buffer.address();
		let material_addr = self.material_storage.buffer_address();

		let cmd = &mut self.cmd_buffers[frame];
		cmd.begin();

		// Cascaded Shadow Map render passes for all active shadow-casting directional lights
		if let Some(sm) = &self.shadow_manager {
			for light_idx in 0..MAX_DIRECTIONAL_LIGHTS {
				if let Some(set) = &sm.cascade_sets[light_idx] {
					for cascade in set.cascades.iter().take(CASCADE_COUNT) {
						let cascade_target = rhi::RenderTarget::builder(SHADOW_MAP_SIZE, SHADOW_MAP_SIZE)
							.depth(&cascade.image, &cascade.view, rhi::Format::Depth32f)
							.build(device);

						cmd.render_pass(&cascade_target, clear_color, |pass| {
							pass.set_viewport_and_scissor(SHADOW_MAP_SIZE, SHADOW_MAP_SIZE);
							pass.bind_pipeline(&sm.pipeline);

							let view_proj = mochi_math::Mat4x4::from_array(cascade.view_proj);

							for packet in &self.shadow_queue.packets {
								let model = mochi_math::Mat4x4::from_array(packet.model_matrix);
								let mvp = (view_proj * model).to_array();

								let pc = ShadowPushConstants {
									mvp,
									vertex_addr: packet.vertex_buffer_addr,
								};

								pass.push_constant(rhi::ShaderStage::Vertex, 0, &pc);

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
			}
		}

		// Clustered Forward compute dispatch (Camera AABBs + Light Culling)
		if let Some(cm) = &mut self.cluster_manager {
			let proj = camera.projection_matrix(aspect);
			let inv_proj = proj.inverse();
			cm.update_camera(cmd, inv_proj.to_array(), camera.near, camera.far);
			cm.cull_lights(cmd, scene_addr);
		}

		if is_post_process {
			if let Some(offscreen) = &self.offscreen_target {
				let mut target_builder = rhi::RenderTarget::builder(width, height)
					.color(
						&offscreen.color_image,
						&offscreen.color_view,
						offscreen.color_format,
					)
					.depth(
						&offscreen.depth_image,
						&offscreen.depth_view,
						offscreen.depth_format,
					);

				if let Some(msaa) = &offscreen.msaa_attachments {
					target_builder = target_builder
						.multisample_color(&msaa.color_image, &msaa.color_view)
						.multisample_depth(&msaa.depth_image, &msaa.depth_view)
						.samples(offscreen.msaa_samples);
				}

				let scene_target = target_builder.build(device);

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

				// On-demand Velocity Buffer generation (only if active mode requires it)
				if self.aa_mode.needs_velocity() {
					if let Some(vm) = &mut self.velocity_manager {
						vm.ensure_target(device, alloc, width, height);
						if let Some(vt) = &vm.target {
							let velocity_rt = rhi::RenderTarget::builder(width, height)
								.color(&vt.image, &vt.view, rhi::Format::V2Float16)
								.build(device);
							let vel_clear = [0.0, 0.0, 0.0, 0.0];
							cmd.render_pass(&velocity_rt, vel_clear, |pass| {
								pass.set_viewport_and_scissor(width, height);
								pass.bind_pipeline(&vm.pipeline);

								let pc = VelocityPushConstants {
									scene_addr,
									depth_tex_id: offscreen.depth_descriptor_id,
									_pad: 0,
								};
								pass.push_constant(rhi::ShaderStage::Pixel, 0, &pc);
								pass.draw(3, 1, 0, 0);
							});
						}
					}
				} else if let Some(vm) = &mut self.velocity_manager {
					vm.free_target();
				}

				if self.aa_mode.is_fxaa() {
					if let (Some(swapchain_target), Some(fxaa_pipeline)) =
						(display.render_target(img_idx), &self.fxaa_pipeline)
					{
						let target_no_depth = swapchain_target.without_depth();
						cmd.render_pass(&target_no_depth, clear_color, |pass| {
							pass.set_viewport_and_scissor(width, height);
							pass.bind_pipeline(fxaa_pipeline);

							let pc = FxaaPushConstants {
								texture_id: offscreen.descriptor_id,
								texel_w: 1.0 / width as f32,
								texel_h: 1.0 / height as f32,
								_pad: 0.0,
							};

							pass.push_constant(rhi::ShaderStage::Pixel, 0, &pc);

							pass.draw(3, 1, 0, 0);
						});
					}
				} else if self.aa_mode.is_smaa()
					&& let Some(smaa) = &mut self.smaa_manager
				{
					let needs_history = self.aa_mode.needs_history();
					smaa.ensure_targets(device, alloc, width, height, needs_history);
					if let Some(targets) = &smaa.targets {
						let rt_metrics = [
							1.0 / width as f32,
							1.0 / height as f32,
							width as f32,
							height as f32,
						];

						// 1. Pass 1: SMAA Edge Detection (Color)
						let edges_rt = rhi::RenderTarget::builder(width, height)
							.color(
								&targets.edges_image,
								&targets.edges_view,
								rhi::Format::V2Norm8U,
							)
							.build(device);
						let edge_clear = [0.0, 0.0, 0.0, 0.0];
						cmd.render_pass(&edges_rt, edge_clear, |pass| {
							pass.set_viewport_and_scissor(width, height);
							pass.bind_pipeline(&smaa.edge_pipeline);

							let pc = SmaaEdgePushConstants {
								rt_metrics,
								color_tex_id: offscreen.descriptor_id,
								_pad0: 0,
								_pad1: 0,
								_pad2: 0,
							};
							pass.push_constant(rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel, 0, &pc);
							pass.draw(3, 1, 0, 0);
						});

						// 2. Pass 2: SMAA Blending Weight Calculation
						let blend_rt = rhi::RenderTarget::builder(width, height)
							.color(
								&targets.blend_image,
								&targets.blend_view,
								rhi::Format::V4Norm8U,
							)
							.build(device);
						let blend_clear = [0.0, 0.0, 0.0, 0.0];
						cmd.render_pass(&blend_rt, blend_clear, |pass| {
							pass.set_viewport_and_scissor(width, height);
							pass.bind_pipeline(&smaa.blend_pipeline);

							let subsample_indices = match self.aa_mode {
								AntiAliasingMode::SmaaT2x | AntiAliasingMode::Smaa4x => {
									if (self.frame_index % 2) == 0 {
										[1.0, 1.0, 1.0, 0.0]
									} else {
										[2.0, 2.0, 2.0, 0.0]
									}
								}
								_ => [0.0, 0.0, 0.0, 0.0],
							};

							let pc = SmaaBlendPushConstants {
								rt_metrics,
								subsample_indices,
								edges_tex_id: targets.edges_descriptor_id,
								area_tex_id: smaa.area_descriptor_id,
								search_tex_id: smaa.search_descriptor_id,
								_pad: 0,
							};
							pass.push_constant(rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel, 0, &pc);
							pass.draw(3, 1, 0, 0);
						});

						let velocity_tex_id = self
							.velocity_manager
							.as_ref()
							.and_then(|v| v.target.as_ref())
							.map(|vt| vt.descriptor_id)
							.unwrap_or(0);

						if needs_history {
							if let (Some(history_images), Some(history_views), Some(history_descs)) = (
								&targets.history_images,
								&targets.history_views,
								&targets.history_descriptor_ids,
							) {
								let curr_idx = (self.frame_index % 2) as usize;
								let prev_idx = 1 - curr_idx;

								// 3. Pass 3: SMAA Neighborhood Blending (Reprojected with Velocity in Alpha)
								let history_rt = rhi::RenderTarget::builder(width, height)
									.color(
										&history_images[curr_idx],
										&history_views[curr_idx],
										smaa.swapchain_color_format,
									)
									.build(device);
								let hist_clear = [0.0, 0.0, 0.0, 0.0];
								cmd.render_pass(&history_rt, hist_clear, |pass| {
									pass.set_viewport_and_scissor(width, height);
									pass.bind_pipeline(&smaa.neighbor_reproject_pipeline);

									let pc = SmaaNeighborPushConstants {
										rt_metrics,
										color_tex_id: offscreen.descriptor_id,
										blend_tex_id: targets.blend_descriptor_id,
										velocity_tex_id,
										_pad: 0,
									};
									pass.push_constant(rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel, 0, &pc);
									pass.draw(3, 1, 0, 0);
								});

								// 4. Pass 4: SMAA Temporal Resolve
								if let Some(swapchain_target) = display.render_target(img_idx) {
									let target_no_depth = swapchain_target.without_depth();
									cmd.render_pass(&target_no_depth, clear_color, |pass| {
										pass.set_viewport_and_scissor(width, height);
										pass.bind_pipeline(&smaa.resolve_pipeline);

										let pc = SmaaResolvePushConstants {
											rt_metrics,
											current_color_tex_id: history_descs[curr_idx],
											previous_color_tex_id: history_descs[prev_idx],
											velocity_tex_id,
											has_history: if smaa.has_history { 1 } else { 0 },
										};
										pass.push_constant(rhi::ShaderStage::Pixel, 0, &pc);
										pass.draw(3, 1, 0, 0);
									});
								}
								smaa.has_history = true;
							}
						} else {
							// 3. Pass 3: SMAA Neighborhood Blending (Direct to Swapchain)
							if let Some(swapchain_target) = display.render_target(img_idx) {
								let target_no_depth = swapchain_target.without_depth();
								cmd.render_pass(&target_no_depth, clear_color, |pass| {
									pass.set_viewport_and_scissor(width, height);
									pass.bind_pipeline(&smaa.neighbor_pipeline);

									let pc = SmaaNeighborPushConstants {
										rt_metrics,
										color_tex_id: offscreen.descriptor_id,
										blend_tex_id: targets.blend_descriptor_id,
										velocity_tex_id: 0,
										_pad: 0,
									};
									pass.push_constant(rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel, 0, &pc);
									pass.draw(3, 1, 0, 0);
								});
							}
						}
					}
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
		pass.set_viewport_and_scissor(width, height);

		let mut currently_bound_pipeline_id: Option<u16> = None;

		for packet in &render_queue.packets {
			let template = packet.material.template();
			let template_id = template.id();

			if currently_bound_pipeline_id != Some(template_id)
				&& let Some(pipeline) = template.pipeline(PassKind::ForwardLit)
			{
				pass.bind_pipeline(&pipeline);
				currently_bound_pipeline_id = Some(template_id);
			}

			let pc = ObjectPushConstants {
				model: packet.model_matrix,
				scene_addr,
				vertex_addr: packet.vertex_buffer_addr,
				material_addr,
				material_id: packet.material_id,
				flags: 0,
			};

			pass.push_constant(rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel, 0, &pc);

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
