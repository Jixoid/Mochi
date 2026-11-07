/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::marker::PhantomData;

use crate::format::Format;
use crate::traits::{RhiBackend, RhiRenderTarget};


#[derive(Clone, Copy)]
pub struct RenderTarget<'a, B: RhiBackend> {
	inner: B::RenderTarget,
	_marker: PhantomData<&'a ()>,
}

unsafe impl<'a, B: RhiBackend> Send for RenderTarget<'a, B> {}
unsafe impl<'a, B: RhiBackend> Sync for RenderTarget<'a, B> {}

impl<'a, B: RhiBackend> RenderTarget<'a, B> {
	pub fn from_raw(inner: B::RenderTarget) -> Self {
		Self {
			inner,
			_marker: PhantomData,
		}
	}

	pub fn raw(&self) -> &B::RenderTarget {
		&self.inner
	}

	pub fn color_format(&self) -> Format {
		self.inner.color_format()
	}

	pub fn depth_format(&self) -> Format {
		self.inner.depth_format()
	}
}
