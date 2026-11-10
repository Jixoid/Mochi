/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;
use std::sync::Arc;
use crate::material::instance::MaterialInstance;
use crate::material::types::PbrMaterialGpuData;
use crate::rhi::{self, AllocationCreate, AllocationLocation, BufferUsage, TransferTime};


pub struct MaterialStorage {
	materials: HashMap<u32, (Arc<MaterialInstance>, usize)>,
	gpu_data: Vec<PbrMaterialGpuData>,
	gpu_buffer: Option<rhi::Buffer>,
	is_dirty: bool,
}

impl MaterialStorage {
	pub fn new() -> Self {
		Self {
			materials: HashMap::new(),
			gpu_data: Vec::new(),
			gpu_buffer: None,
			is_dirty: false,
		}
	}

	pub fn register(&mut self, material: Arc<MaterialInstance>) -> u32 {
		let id = material.id();
		if let Some((_, slot)) = self.materials.get(&id) {
			return *slot as u32;
		}

		let slot = self.gpu_data.len();
		self.gpu_data.push(material.to_gpu_data());
		self.materials.insert(id, (material, slot));
		self.is_dirty = true;
		slot as u32
	}

	pub fn get_slot(&self, id: u32) -> Option<u32> {
		self.materials.get(&id).map(|(_, slot)| *slot as u32)
	}

	pub fn update_gpu(&mut self, device: &rhi::Device, alloc: &rhi::Allocator) {
		// Update data from registered material instances
		for (mat, slot) in self.materials.values() {
			let current = mat.to_gpu_data();
			if self.gpu_data[*slot] != current {
				self.gpu_data[*slot] = current;
				self.is_dirty = true;
			}
		}

		if self.gpu_data.is_empty() {
			return;
		}

		let required_size = (std::mem::size_of::<PbrMaterialGpuData>() * self.gpu_data.len().max(16)) as u64;

		let needs_realloc = self.gpu_buffer.as_ref().map_or(true, |b| b.size() < required_size);

		if needs_realloc {
			let new_buffer = rhi::Buffer::new(
				alloc,
				required_size,
				BufferUsage::StorageBuffer | BufferUsage::DeviceAddress,
				AllocationCreate::Mapped,
				AllocationLocation::Auto,
			);
			self.gpu_buffer = Some(new_buffer);
			self.is_dirty = true;
		}

		if self.is_dirty {
			if let Some(buffer) = &mut self.gpu_buffer {
				let uploader = rhi::Uploader::new(device);
				let bytes = unsafe {
					std::slice::from_raw_parts(
						self.gpu_data.as_ptr() as *const u8,
						std::mem::size_of_val(&self.gpu_data[..]),
					)
				};
				uploader.copy_memory_to_buffer(TransferTime::Now, bytes, buffer);
			}
			self.is_dirty = false;
		}
	}

	pub fn buffer_address(&self) -> u64 {
		self.gpu_buffer.as_ref().map_or(0, |b| b.address())
	}
}
