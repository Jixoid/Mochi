/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use crate::material::{BlendMode, MaterialInstance, MaterialStorage, MaterialTemplate, PassKind, TextureManager};
use crate::rendering::frame::SceneFrameBuffer;
use crate::rendering::queue::RenderQueue;
use crate::rhi;
use crate::scene::{CameraComponent, Scene, TransformComponent};
use mochi_except::*;


static PBR_VERT_SPV: &[u8] = include_bytes!("../../res/shaders/pbr.vert.spv");
static PBR_FRAG_SPV: &[u8] = include_bytes!("../../res/shaders/pbr.frag.spv");


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

		// Update frame scene buffer (Camera + Dynamic Lights)
		self.frame_buffer.update(
			device,
			alloc,
			scene,
			&cam_transform,
			&camera,
			aspect,
			time,
			delta_time,
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

		let target = display.render_target(img_idx);
		let cmd = &mut self.cmd_buffers[frame];
		cmd.begin();

		if let Some(target) = target {
			cmd.render_pass(&target, clear_color, |pass| {
				pass.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
				pass.set_scissor(0, 0, width, height);

				let mut currently_bound_pipeline_id: Option<u16> = None;
				let scene_addr = self.frame_buffer.address();
				let material_addr = self.material_storage.buffer_address();

				for packet in &self.render_queue.packets {
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
			});
		}

		cmd.end();

		let sig_sem = display.render_finished_semaphore(img_idx);
		let fence = sync.in_flight_fence();
		cmdmng.submit_with_sync(cmd, Some(&wait_sem), Some(&sig_sem), Some(&fence));
		sync.end_frame();

		display.present_with_semaphore(img_idx, &sig_sem);
	}
}
