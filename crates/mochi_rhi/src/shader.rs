/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use bitflags::bitflags;

use crate::manager::device::Device;
use crate::traits::{RhiBackend, RhiDevice};


bitflags! {
	#[derive(Copy, Clone, Debug, PartialEq, Eq)]
	pub struct ShaderStage: u32 {
		const Vertex       = 0x00000001;
		const Pixel        = 0x00000010;
		const Compute      = 0x00000020;
		const Task         = 0x00000040;
		const Mesh         = 0x00000080;
		const RayGen       = 0x00000100;
		const AnyHit       = 0x00000200;
		const ClosestHit   = 0x00000400;
		const Miss         = 0x00000800;
		const Intersection = 0x00001000;
		const Callable     = 0x00002000;
	}
}


#[derive(Clone)]
pub struct Shader<B: RhiBackend> {
	inner: Arc<B::Shader>,
	stage: ShaderStage,
	entry: String,
}

unsafe impl<B: RhiBackend> Send for Shader<B> {}
unsafe impl<B: RhiBackend> Sync for Shader<B> {}

impl<B: RhiBackend> Shader<B> {
	pub fn new(device: &Device<B>, stage: ShaderStage, code: &[u8], entry: &str) -> Self {
		let sh = device
			.raw()
			.create_shader(stage, code, entry)
			.expect("Failed to create shader");
		Self {
			inner: Arc::new(sh),
			stage,
			entry: entry.to_string(),
		}
	}

	pub fn raw(&self) -> &B::Shader {
		&self.inner
	}

	pub fn stage(&self) -> ShaderStage {
		self.stage
	}

	pub fn entry(&self) -> &str {
		&self.entry
	}
}
