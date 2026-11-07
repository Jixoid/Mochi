/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use ash::vk;

use mochi_rhi::traits::RhiSampler;


pub struct SamplerInner {
	device: ash::Device,
	handle: vk::Sampler,
}

unsafe impl Send for SamplerInner {}
unsafe impl Sync for SamplerInner {}

impl Drop for SamplerInner {
	fn drop(&mut self) {
		if self.handle != vk::Sampler::null() {
			unsafe {
				self.device.destroy_sampler(self.handle, None);
			}
			self.handle = vk::Sampler::null();
		}
	}
}

#[derive(Clone)]
pub struct AshSampler {
	inner: Arc<SamplerInner>,
}

unsafe impl Send for AshSampler {}
unsafe impl Sync for AshSampler {}

impl AshSampler {
	pub fn new(device: &ash::Device, handle: vk::Sampler) -> Self {
		Self {
			inner: Arc::new(SamplerInner {
				device: device.clone(),
				handle,
			}),
		}
	}

	pub fn handle(&self) -> vk::Sampler {
		self.inner.handle
	}
}

impl RhiSampler for AshSampler {}
