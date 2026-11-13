/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::marker::PhantomData;

use crate::format::Format;
use crate::image::{Image2, ImageView2};
use crate::manager::device::Device;
use crate::traits::{RhiBackend, RhiDevice, RhiRenderTarget};
use crate::types::MsaaSamples;


#[derive(Clone, Debug)]
pub struct RenderTargetDesc<'a, B: RhiBackend> {
	pub width: u32,
	pub height: u32,
	pub color_image: Option<&'a B::Image2>,
	pub color_view: Option<&'a B::ImageView2>,
	pub color_format: Format,
	pub resolve_color_image: Option<&'a B::Image2>,
	pub resolve_color_view: Option<&'a B::ImageView2>,
	pub depth_image: Option<&'a B::Image2>,
	pub depth_view: Option<&'a B::ImageView2>,
	pub depth_format: Format,
	pub resolve_depth_image: Option<&'a B::Image2>,
	pub resolve_depth_view: Option<&'a B::ImageView2>,
	pub samples: MsaaSamples,
}


#[derive(Clone, Copy)]
pub struct RenderTarget<'a, B: RhiBackend> {
	inner: B::RenderTarget,
	_marker: PhantomData<&'a ()>,
}

unsafe impl<'a, B: RhiBackend> Send for RenderTarget<'a, B> {}
unsafe impl<'a, B: RhiBackend> Sync for RenderTarget<'a, B> {}

impl<'a, B: RhiBackend> RenderTarget<'a, B> {
	pub fn builder(width: u32, height: u32) -> RenderTargetBuilder<'a, B> {
		RenderTargetBuilder::new(width, height)
	}

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

	pub fn without_depth(&self) -> Self {
		Self::from_raw(self.inner.without_depth())
	}
}


pub struct RenderTargetBuilder<'a, B: RhiBackend> {
	width: u32,
	height: u32,
	color_image: Option<&'a Image2<B>>,
	color_view: Option<&'a ImageView2<B>>,
	color_format: Format,
	msaa_color_image: Option<&'a Image2<B>>,
	msaa_color_view: Option<&'a ImageView2<B>>,
	depth_image: Option<&'a Image2<B>>,
	depth_view: Option<&'a ImageView2<B>>,
	depth_format: Format,
	msaa_depth_image: Option<&'a Image2<B>>,
	msaa_depth_view: Option<&'a ImageView2<B>>,
	samples: MsaaSamples,
}

impl<'a, B: RhiBackend> RenderTargetBuilder<'a, B> {
	pub fn new(width: u32, height: u32) -> Self {
		Self {
			width,
			height,
			color_image: None,
			color_view: None,
			color_format: Format::Undefined,
			msaa_color_image: None,
			msaa_color_view: None,
			depth_image: None,
			depth_view: None,
			depth_format: Format::Undefined,
			msaa_depth_image: None,
			msaa_depth_view: None,
			samples: MsaaSamples::Off,
		}
	}

	pub fn color(mut self, image: &'a Image2<B>, view: &'a ImageView2<B>, format: Format) -> Self {
		self.color_image = Some(image);
		self.color_view = Some(view);
		self.color_format = format;
		self
	}

	pub fn multisample_color(mut self, image: &'a Image2<B>, view: &'a ImageView2<B>) -> Self {
		self.msaa_color_image = Some(image);
		self.msaa_color_view = Some(view);
		self
	}

	pub fn depth(mut self, image: &'a Image2<B>, view: &'a ImageView2<B>, format: Format) -> Self {
		self.depth_image = Some(image);
		self.depth_view = Some(view);
		self.depth_format = format;
		self
	}

	pub fn multisample_depth(mut self, image: &'a Image2<B>, view: &'a ImageView2<B>) -> Self {
		self.msaa_depth_image = Some(image);
		self.msaa_depth_view = Some(view);
		self
	}

	pub fn samples(mut self, samples: MsaaSamples) -> Self {
		self.samples = samples;
		self
	}

	pub fn build(self, device: &Device<B>) -> RenderTarget<'a, B> {
		let (color_image, color_view, resolve_color_image, resolve_color_view, samples) =
			if let (Some(msaa_img), Some(msaa_view)) = (self.msaa_color_image, self.msaa_color_view) {
				let res_img = self.color_image.map(|i| i.raw());
				let res_view = self.color_view.map(|v| v.raw());
				(Some(msaa_img.raw()), Some(msaa_view.raw()), res_img, res_view, self.samples)
			} else {
				(self.color_image.map(|i| i.raw()), self.color_view.map(|v| v.raw()), None, None, MsaaSamples::Off)
			};

		let (depth_image, depth_view, resolve_depth_image, resolve_depth_view) =
			if let (Some(msaa_depth_img), Some(msaa_depth_view)) = (self.msaa_depth_image, self.msaa_depth_view) {
				let res_depth_img = self.depth_image.map(|i| i.raw());
				let res_depth_view = self.depth_view.map(|v| v.raw());
				(Some(msaa_depth_img.raw()), Some(msaa_depth_view.raw()), res_depth_img, res_depth_view)
			} else {
				(self.depth_image.map(|i| i.raw()), self.depth_view.map(|v| v.raw()), None, None)
			};

		let desc = RenderTargetDesc {
			width: self.width,
			height: self.height,
			color_image,
			color_view,
			color_format: self.color_format,
			resolve_color_image,
			resolve_color_view,
			depth_image,
			depth_view,
			depth_format: self.depth_format,
			resolve_depth_image,
			resolve_depth_view,
			samples,
		};

		let inner = device.raw().create_render_target(&desc);
		RenderTarget::from_raw(inner)
	}
}
