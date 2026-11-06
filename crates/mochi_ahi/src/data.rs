/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


#[derive(Clone, Debug)]
pub struct AudioData {
	pub sample_rate: u32,
	pub channels: u8,
	pub pcm_data: Vec<f32>,
}

impl AudioData {
	pub fn new(sample_rate: u32, channels: u8, pcm_data: Vec<f32>) -> Self {
		Self {
			sample_rate,
			channels,
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
