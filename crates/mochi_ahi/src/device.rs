/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::data::AudioData;
use crate::error::AhiResult;
use crate::traits::{AhiBackend, AhiDevice};


/// Safe generic audio device wrapper parameterized by an AHI backend
pub struct Device<B: AhiBackend> {
	inner: B::Device,
}

impl<B: AhiBackend> Device<B> {
	pub fn new(app_name: &str, major: u16, minor: u16, patch: u16, build: u16) -> AhiResult<Self> {
		let inner = B::create_device(app_name, major, minor, patch, build)?;
		Ok(Self { inner })
	}

	pub fn sample_rate(&self) -> u32 {
		self.inner.sample_rate()
	}

	pub fn channels(&self) -> u16 {
		self.inner.channels()
	}

	pub fn set_callback<F: FnMut(&mut [f32], u32) + Send + 'static>(&mut self, callback: F) {
		self.inner.set_callback(Box::new(callback));
	}

	pub fn start(&mut self) -> bool {
		self.inner.start()
	}

	pub fn stop(&mut self) {
		self.inner.stop();
	}

	pub fn is_playing(&self) -> bool {
		self.inner.is_playing()
	}

	pub fn decode_file(&self, path: &str) -> AhiResult<AudioData> {
		self.inner.decode_file(path)
	}

	pub fn decode_memory(&self, bytes: &[u8]) -> AhiResult<AudioData> {
		self.inner.decode_memory(bytes)
	}
}
