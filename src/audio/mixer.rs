/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};
use mochi_math::Vec3;
use crate::audio::AudioSource;


#[derive(Copy, Clone, Debug)]
pub struct Listener {
	pub position: Vec3<f32>,
	pub forward: Vec3<f32>,
	pub up: Vec3<f32>,
}

impl Listener {
	pub fn new() -> Self {
		Self {
			position: Vec3::ZERO,
			forward: Vec3::FORWARD,
			up: Vec3::UP,
		}
	}
}

impl Default for Listener {
	fn default() -> Self {
		Self::new()
	}
}


pub struct Mixer {
	sources: Vec<Arc<Mutex<AudioSource>>>,
	listener: Listener,
	master_volume: f32,
}

impl Mixer {
	pub fn new() -> Self {
		Self {
			sources: Vec::new(),
			listener: Listener::new(),
			master_volume: 1.0,
		}
	}

	pub fn set_listener(&mut self, listener: Listener) {
		self.listener = listener;
	}

	pub fn listener(&self) -> &Listener {
		&self.listener
	}

	pub fn set_master_volume(&mut self, volume: f32) {
		self.master_volume = volume.clamp(0.0, 2.0);
	}

	pub fn master_volume(&self) -> f32 {
		self.master_volume
	}

	pub fn add_source(&mut self, source: Arc<Mutex<AudioSource>>) {
		if !self.sources.iter().any(|s| Arc::ptr_eq(s, &source)) {
			self.sources.push(source);
		}
	}

	pub fn remove_source(&mut self, source: &Arc<Mutex<AudioSource>>) {
		self.sources.retain(|s| !Arc::ptr_eq(s, source));
	}

	pub fn remove_stopped_sources(&mut self) {
		self.sources.retain(|s| {
			if let Ok(src) = s.lock() {
				src.is_playing() || src.is_paused()
			} else {
				false
			}
		});
	}

	pub fn source_count(&self) -> usize {
		self.sources.len()
	}

	pub fn process(&mut self, output_buffer: &mut [f32], frame_count: u32) {
		// Zero the output stereo buffer first
		output_buffer.fill(0.0);

		if self.master_volume <= 0.0001 {
			return;
		}

		let right = self.listener.forward.cross(&self.listener.up).normalize();

		for source_arc in &self.sources {
			let mut source = match source_arc.lock() {
				Ok(s) => s,
				Err(_) => continue,
			};

			if !source.is_playing() {
				continue;
			}

			// Relative position
			let rel_pos = source.position - self.listener.position;
			let dist = rel_pos.length();

			// Distance attenuation
			let mut attenuation = 1.0f32;
			if dist > source.min_distance {
				if dist >= source.max_distance {
					attenuation = 0.0;
				} else {
					let range = source.max_distance - source.min_distance;
					if range > 0.0001 {
						attenuation = (1.0 - (dist - source.min_distance) / range).clamp(0.0, 1.0);
					}
				}
			}

			// Panning: -1.0 (Left) to 1.0 (Right)
			let mut pan = 0.0f32;
			if dist > 0.0001 {
				pan = (rel_pos / dist).dot(&right).clamp(-1.0, 1.0);
			}

			// Volume multipliers
			let vol_l = source.volume * self.master_volume * attenuation * (1.0 - pan).clamp(0.0, 1.0);
			let vol_r = source.volume * self.master_volume * attenuation * (1.0 + pan).clamp(0.0, 1.0);

			if vol_l > 0.001 || vol_r > 0.001 {
				source.mix_frames(output_buffer, frame_count, vol_l, vol_r);
			}
		}
	}
}

impl Default for Mixer {
	fn default() -> Self {
		Self::new()
	}
}
