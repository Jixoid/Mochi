/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::ffi::*;


#[derive(Clone, Copy)]
pub struct RenderTarget {
	handle: MochiRenderTargetHandle,
}

impl RenderTarget {
	pub fn from_handle(handle: MochiRenderTargetHandle) -> Self {
		Self { handle }
	}

	pub fn handle(&self) -> MochiRenderTargetHandle {
		self.handle
	}


	pub fn color_format_raw(&self) -> u32 {
		unsafe { mochi_rhi_render_target_get_color_format(self.handle) }
	}

	pub fn depth_format_raw(&self) -> u32 {
		unsafe { mochi_rhi_render_target_get_depth_format(self.handle) }
	}

	
	pub fn color_format(&self) -> crate::Format {
		crate::Format::from_raw(self.color_format_raw())
	}

	pub fn depth_format(&self) -> crate::Format {
		crate::Format::from_raw(self.depth_format_raw())
	}
}
