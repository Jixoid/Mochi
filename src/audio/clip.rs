/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_ahi::{AudioData, Device};


#[derive(Clone, Debug)]
pub struct AudioClip {
	pub sample_rate: u32,
	pub channels: u8,
	pub pcm_data: Vec<f32>,
}

impl AudioClip {
	pub fn new(sample_rate: u32, channels: u8, pcm_data: Vec<f32>) -> Self {
		Self {
			sample_rate,
			channels,
			pcm_data,
		}
	}

	pub fn from_data(data: AudioData) -> Self {
		Self {
			sample_rate: data.sample_rate,
			channels: data.channels,
			pcm_data: data.pcm_data,
		}
	}

	pub fn from_file(device: &Device, path: &str) -> Result<Self, String> {
		let data = device.decode_file(path)?;
		Ok(Self::from_data(data))
	}

	pub fn from_memory(device: &Device, bytes: &[u8]) -> Result<Self, String> {
		let data = device.decode_memory(bytes)?;
		Ok(Self::from_data(data))
	}

	pub fn sine_wave(frequency: f32, duration_secs: f32, sample_rate: u32) -> Self {
		let total_frames = (sample_rate as f32 * duration_secs) as usize;
		let mut pcm_data = Vec::with_capacity(total_frames * 2);

		for frame in 0..total_frames {
			let t = frame as f32 / sample_rate as f32;
			let sample = (t * frequency * 2.0 * std::f32::consts::PI).sin();
			// Stereo (L + R)
			pcm_data.push(sample);
			pcm_data.push(sample);
		}

		Self {
			sample_rate,
			channels: 2,
			pcm_data,
		}
	}

	pub fn frame_count(&self) -> usize {
		if self.channels == 0 {
			0
		} else {
			self.pcm_data.len() / self.channels as usize
		}
	}

	pub fn duration_seconds(&self) -> f32 {
		if self.sample_rate == 0 {
			0.0
		} else {
			self.frame_count() as f32 / self.sample_rate as f32
		}
	}
}
