/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use mochi_math::Vec3;
use crate::audio::AudioClip;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SourceState {
	Stopped,
	Playing,
	Paused,
}

pub struct AudioSource {
	clip: Option<Arc<AudioClip>>,
	cursor: usize,
	state: SourceState,
	looping: bool,
	pub position: Vec3<f32>,
	pub volume: f32,
	pub pitch: f32,
	pub min_distance: f32,
	pub max_distance: f32,
}

impl AudioSource {
	pub fn new() -> Self {
		Self {
			clip: None,
			cursor: 0,
			state: SourceState::Stopped,
			looping: false,
			position: Vec3::ZERO,
			volume: 1.0,
			pitch: 1.0,
			min_distance: 1.0,
			max_distance: 100.0,
		}
	}

	pub fn with_clip(clip: Arc<AudioClip>) -> Self {
		let mut source = Self::new();
		source.bind_clip(clip);
		source
	}

	pub fn bind_clip(&mut self, clip: Arc<AudioClip>) {
		self.clip = Some(clip);
		self.cursor = 0;
	}

	pub fn clip(&self) -> Option<&Arc<AudioClip>> {
		self.clip.as_ref()
	}

	pub fn state(&self) -> SourceState {
		self.state
	}

	pub fn is_playing(&self) -> bool {
		self.state == SourceState::Playing
	}

	pub fn is_paused(&self) -> bool {
		self.state == SourceState::Paused
	}

	pub fn is_stopped(&self) -> bool {
		self.state == SourceState::Stopped
	}

	pub fn play(&mut self) {
		self.state = SourceState::Playing;
	}

	pub fn pause(&mut self) {
		self.state = SourceState::Paused;
	}

	pub fn stop(&mut self) {
		self.state = SourceState::Stopped;
		self.cursor = 0;
	}

	pub fn set_looping(&mut self, looping: bool) {
		self.looping = looping;
	}

	pub fn is_looping(&self) -> bool {
		self.looping
	}

	pub fn set_volume(&mut self, volume: f32) {
		self.volume = volume.max(0.0);
	}

	pub fn set_position(&mut self, position: impl Into<Vec3<f32>>) {
		self.position = position.into();
	}

	pub fn set_distance_range(&mut self, min: f32, max: f32) {
		self.min_distance = min.max(0.0);
		self.max_distance = max.max(self.min_distance);
	}

	pub fn mix_frames(
		&mut self,
		mix_buffer: &mut [f32],
		frame_count: u32,
		vol_l: f32,
		vol_r: f32,
	) -> u32 {
		if self.state != SourceState::Playing {
			return 0;
		}

		let clip = match &self.clip {
			Some(c) => c,
			None => return 0,
		};

		let total_frames = clip.frame_count();
		if total_frames == 0 {
			return 0;
		}

		let channels = clip.channels as usize;
		let mut frames_mixed: usize = 0;
		let target_frames = frame_count as usize;

		while frames_mixed < target_frames && self.state == SourceState::Playing {
			if self.cursor >= total_frames {
				if self.looping {
					self.cursor = 0;
				} else {
					self.stop();
					break;
				}
			}

			let chunk_size = (target_frames - frames_mixed).min(total_frames - self.cursor);
			if chunk_size == 0 {
				break;
			}

			if channels == 1 {
				for i in 0..chunk_size {
					let sample = clip.pcm_data[self.cursor + i];
					let out_idx = (frames_mixed + i) * 2;
					mix_buffer[out_idx] += sample * vol_l;
					mix_buffer[out_idx + 1] += sample * vol_r;
				}
			} else if channels == 2 {
				for i in 0..chunk_size {
					let src_idx = (self.cursor + i) * 2;
					let out_idx = (frames_mixed + i) * 2;
					mix_buffer[out_idx] += clip.pcm_data[src_idx] * vol_l;
					mix_buffer[out_idx + 1] += clip.pcm_data[src_idx + 1] * vol_r;
				}
			}

			self.cursor += chunk_size;
			frames_mixed += chunk_size;

			if self.cursor >= total_frames {
				if self.looping {
					self.cursor = 0;
				} else {
					self.stop();
					break;
				}
			}
		}

		frames_mixed as u32
	}
}

impl Default for AudioSource {
	fn default() -> Self {
		Self::new()
	}
}
