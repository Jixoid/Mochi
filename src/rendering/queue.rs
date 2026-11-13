/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use crate::material::{BlendMode, MaterialInstance, MaterialStorage, PassKind};
use crate::scene::{Mesh, MeshRendererComponent, Scene, TransformComponent};
use mochi_math::Vec3;


pub struct RenderPacket {
	pub sort_key: u64,
	pub model_matrix: [f32; 16],
	pub vertex_buffer_addr: u64,
	pub index_buffer_addr: u64,
	pub index_count: u32,
	pub vertex_count: u32,
	pub material_id: u32,
	pub material: Arc<MaterialInstance>,
	pub mesh: Arc<Mesh>,
	pub is_indexed: bool,
}

pub struct RenderQueue {
	pub packets: Vec<RenderPacket>,
}

impl RenderQueue {
	pub fn new() -> Self {
		Self {
			packets: Vec::with_capacity(128),
		}
	}

	pub fn clear(&mut self) {
		self.packets.clear();
	}

	pub fn extract_from_scene(
		&mut self,
		scene: &Scene,
		pass: PassKind,
		camera_pos: Vec3<f32>,
		material_storage: &mut MaterialStorage,
	) {
		self.clear();

		for (_entity, (transform, renderer)) in scene
			.world()
			.query::<(&TransformComponent, &MeshRendererComponent)>()
			.iter()
		{
			if !renderer.is_visible {
				continue;
			}

			let mesh = &renderer.mesh;
			let material = &renderer.material;

			let is_transparent = matches!(
				material.blend_mode(),
				BlendMode::Transparent | BlendMode::Additive
			);

			// Filter by pass requirements
			match pass {
				PassKind::DepthPrepass => {
					if is_transparent {
						continue;
					}
				}
				PassKind::ShadowCaster => {
					if !renderer.cast_shadows || is_transparent {
						continue;
					}
				}
				PassKind::ForwardLit => {}
			}

			// Register material in storage and obtain its slot ID
			let slot_id = material_storage.register(Arc::clone(material));

			let depth = transform.translation.dist(&camera_pos).clamp(0.0, 1000.0);

			let sort_key = Self::build_sort_key(
				pass,
				material.render_priority(),
				material.template().id(),
				slot_id,
				depth,
				is_transparent,
			);

			let model_matrix = transform.to_array();

			self.packets.push(RenderPacket {
				sort_key,
				model_matrix,
				vertex_buffer_addr: mesh.vertex_address(),
				index_buffer_addr: mesh.index_address(),
				index_count: mesh.index_count(),
				vertex_count: mesh.vertex_count(),
				material_id: slot_id,
				material: Arc::clone(material),
				mesh: Arc::clone(mesh),
				is_indexed: mesh.is_indexed(),
			});
		}

		self.sort();
	}

	pub fn sort(&mut self) {
		self.packets.sort_unstable_by_key(|p| p.sort_key);
	}

	pub fn build_sort_key(
		pass: PassKind,
		render_priority: i32,
		pipeline_id: u16,
		material_id: u32,
		depth: f32,
		is_transparent: bool,
	) -> u64 {
		let pass_bits = match pass {
			PassKind::DepthPrepass => 0u64,
			PassKind::ShadowCaster => 1u64,
			PassKind::ForwardLit if !is_transparent => 2u64,
			PassKind::ForwardLit => 3u64,
		} & 0xF;

		let layer_bits = ((render_priority.clamp(-128, 127) + 128) as u64) & 0xFF;
		let pipeline_bits = (pipeline_id as u64) & 0xFFFF;
		let mat_bits = (material_id as u64) & 0xFFFF;

		// 20-bit quantized depth (0..1048575)
		let normalized_depth = (depth / 1000.0).clamp(0.0, 1.0);
		let raw_depth_quantized = (normalized_depth * 1048575.0) as u64;

		let depth_bits = if is_transparent {
			// Back-to-Front (highest depth first)
			1048575 - raw_depth_quantized
		} else {
			// Front-to-Back (lowest depth first) for Early-Z
			raw_depth_quantized
		} & 0xFFFFF;

		(pass_bits << 60) | (layer_bits << 52) | (pipeline_bits << 36) | (mat_bits << 20) | depth_bits
	}
}

impl Default for RenderQueue {
	fn default() -> Self {
		Self::new()
	}
}
