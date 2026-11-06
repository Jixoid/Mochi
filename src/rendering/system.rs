/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_rhi as rhi;
use mochi_math::Vec4;

static VERT_SPV: &[u8] = include_bytes!("../../res/shaders/base.vert.spv");
static FRAG_SPV: &[u8] = include_bytes!("../../res/shaders/bare.frag.spv");


#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
	pub pos: Vec4<f32>,
	pub normal: Vec4<f32>,
	pub color: Vec4<f32>,
	pub uv: Vec4<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CameraData {
	pub view: [f32; 16],
	pub proj: [f32; 16],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PushConstantData {
	pub model: [f32; 12],
	pub vertex_addr: u64,
	pub camera_addr: u64,
	pub light_addr: u64,
	pub texture_id: u32,
	pub _pad: u32,
}


pub struct RenderingSystem {
	cmd_buffers: Vec<rhi::Command>,
	pipeline: Option<rhi::Pipeline>,
	_meta: Option<rhi::PipelineMeta>,
	_vert_shader: Option<rhi::Shader>,
	_frag_shader: Option<rhi::Shader>,
	vertex_buffer: Option<rhi::Buffer>,
	camera_ubo: Option<rhi::Buffer>,
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
			pipeline: None,
			_meta: None,
			_vert_shader: None,
			_frag_shader: None,
			vertex_buffer: None,
			camera_ubo: None,
		})
	}

	pub fn init_pipeline(
		&mut self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		color_format: rhi::Format,
		depth_format: rhi::Format,
	) {
		println!("[RenderingSystem] Initializing graphics pipeline with BDA & Dynamic Rendering...");

		let vert = rhi::Shader::new(device, rhi::ShaderStage::Vertex, VERT_SPV, "main");
		let frag = rhi::Shader::new(device, rhi::ShaderStage::Pixel, FRAG_SPV, "main");

		let meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel,
			std::mem::size_of::<PushConstantData>() as u32,
		);

		let pipeline = rhi::Pipeline::new_graphics(
			device,
			&meta,
			&[&vert, &frag],
			rhi::PolygonMode::Fill,
			rhi::PrimitiveTopology::TriangleList,
			color_format,
			depth_format,
		);

		// Triangle vertices with counter-clockwise winding
		let vertices = [
			Vertex {
				pos: Vec4::new_with(0.0, -0.5, 0.0, 0.0),
				normal: Vec4::new_with(0.0, 0.0, 1.0, 0.0),
				color: Vec4::new_with(1.0, 0.2, 0.2, 1.0), // Red
				uv: Vec4::new_with(0.5, 0.0, 0.0, 0.0),
			},
			Vertex {
				pos: Vec4::new_with(-0.5, 0.5, 0.0, 0.0),
				normal: Vec4::new_with(0.0, 0.0, 1.0, 0.0),
				color: Vec4::new_with(0.2, 1.0, 0.2, 1.0), // Green
				uv: Vec4::new_with(0.0, 1.0, 0.0, 0.0),
			},
			Vertex {
				pos: Vec4::new_with(0.5, 0.5, 0.0, 0.0),
				normal: Vec4::new_with(0.0, 0.0, 1.0, 0.0),
				color: Vec4::new_with(0.2, 0.4, 1.0, 1.0), // Blue
				uv: Vec4::new_with(1.0, 1.0, 0.0, 0.0),
			},
		];

		let uploader = rhi::Uploader::new(device);

		let vertex_buffer = rhi::Buffer::new(
			alloc,
			(std::mem::size_of::<Vertex>() * vertices.len()) as u64,
			rhi::BufferUsage::DeviceAddress | rhi::BufferUsage::StorageBuffer,
			rhi::AllocationCreate::Mapped,
			rhi::AllocationLocation::Auto,
		);

		let vertex_bytes = unsafe {
			std::slice::from_raw_parts(
				vertices.as_ptr() as *const u8,
				std::mem::size_of_val(&vertices),
			)
		};
		uploader.copy_memory_to_buffer(rhi::TransferTime::Now, vertex_bytes, &vertex_buffer);

		let camera_data = CameraData {
			view: [
				1.0, 0.0, 0.0, 0.0,
				0.0, 1.0, 0.0, 0.0,
				0.0, 0.0, 1.0, 0.0,
				0.0, 0.0, 0.0, 1.0,
			],
			proj: [
				1.0, 0.0, 0.0, 0.0,
				0.0, 1.0, 0.0, 0.0,
				0.0, 0.0, 1.0, 0.0,
				0.0, 0.0, 0.0, 1.0,
			],
		};

		let camera_ubo = rhi::Buffer::new(
			alloc,
			std::mem::size_of::<CameraData>() as u64,
			rhi::BufferUsage::DeviceAddress | rhi::BufferUsage::StorageBuffer | rhi::BufferUsage::UniformBuffer,
			rhi::AllocationCreate::Mapped,
			rhi::AllocationLocation::Auto,
		);

		let camera_bytes = unsafe {
			std::slice::from_raw_parts(
				&camera_data as *const CameraData as *const u8,
				std::mem::size_of::<CameraData>(),
			)
		};
		uploader.copy_memory_to_buffer(rhi::TransferTime::Now, camera_bytes, &camera_ubo);

		println!(
			"[RenderingSystem] Pipeline ready. Vertex buffer address: 0x{:x}, Camera UBO address: 0x{:x}",
			vertex_buffer.address(),
			camera_ubo.address()
		);

		self.pipeline = Some(pipeline);
		self._meta = Some(meta);
		self._vert_shader = Some(vert);
		self._frag_shader = Some(frag);
		self.vertex_buffer = Some(vertex_buffer);
		self.camera_ubo = Some(camera_ubo);
	}

	pub fn begin_frame(&self, sync: &rhi::Synchronizer) -> &rhi::Command {
		sync.begin_frame();
		let frame = sync.current_frame() as usize;
		let cmd = &self.cmd_buffers[frame];
		cmd.begin();
		cmd
	}

	pub fn begin_pass(&self, cmd: &rhi::Command, target: &rhi::RenderTarget, clear_color: impl Into<Vec4<f32>>) {
		cmd.begin_rendering(target, clear_color.into());
	}

	pub fn draw_triangle(&self, cmd: &rhi::Command, width: u32, height: u32) {
		if let (Some(pipeline), Some(vbuf), Some(cambuf)) = (
			&self.pipeline,
			&self.vertex_buffer,
			&self.camera_ubo,
		) {
			cmd.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
			cmd.set_scissor(0, 0, width, height);
			cmd.bind_pipeline(pipeline);

			let pc = PushConstantData {
				model: [
					1.0, 0.0, 0.0, 0.0,
					0.0, 1.0, 0.0, 0.0,
					0.0, 0.0, 1.0, 0.0,
				],
				vertex_addr: vbuf.address(),
				camera_addr: cambuf.address(),
				light_addr: 0,
				texture_id: 0,
				_pad: 0,
			};

			cmd.push_constant(
				rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel,
				0,
				&pc,
			);

			cmd.draw(3, 1, 0, 0);
		}
	}

	pub fn end_pass(&self, cmd: &rhi::Command, target: &rhi::RenderTarget) {
		cmd.end_rendering(target);
	}

	pub fn end_frame(
		&self,
		cmd: &rhi::Command,
		cmdmng: &rhi::CommandManager,
		syncer: &rhi::Synchronizer,
		wait_sem: *mut std::ffi::c_void,
		signal_sem: *mut std::ffi::c_void,
	) {
		cmd.end();
		cmdmng.submit_with_sync(cmd, wait_sem, signal_sem, syncer.in_flight_fence());
		syncer.end_frame();
	}
}
