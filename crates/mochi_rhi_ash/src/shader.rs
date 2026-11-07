/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use ash::vk;

use mochi_rhi::shader::ShaderStage;
use mochi_rhi::traits::RhiShader;


pub struct ShaderInner {
	device: ash::Device,
	handle: vk::ShaderModule,
	stage: ShaderStage,
	entry: String,
}

unsafe impl Send for ShaderInner {}
unsafe impl Sync for ShaderInner {}

impl Drop for ShaderInner {
	fn drop(&mut self) {
		if self.handle != vk::ShaderModule::null() {
			unsafe {
				self.device.destroy_shader_module(self.handle, None);
			}
			self.handle = vk::ShaderModule::null();
		}
	}
}

#[derive(Clone)]
pub struct AshShader {
	inner: Arc<ShaderInner>,
}

unsafe impl Send for AshShader {}
unsafe impl Sync for AshShader {}

impl AshShader {
	pub fn new(device: &ash::Device, handle: vk::ShaderModule, stage: ShaderStage, entry: &str) -> Self {
		Self {
			inner: Arc::new(ShaderInner {
				device: device.clone(),
				handle,
				stage,
				entry: entry.to_string(),
			}),
		}
	}

	pub fn handle(&self) -> vk::ShaderModule {
		self.inner.handle
	}
}

impl RhiShader for AshShader {
	fn stage(&self) -> ShaderStage {
		self.inner.stage
	}

	fn entry(&self) -> &str {
		&self.inner.entry
	}
}
