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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::audio::AudioClip;

	#[test]
	fn test_mixer_spatial_panning_and_attenuation() {
		let pcm = vec![1.0; 200]; // 100 frames stereo of constant 1.0
		let clip = Arc::new(AudioClip::new(44100, 2, pcm));

		let mut mixer = Mixer::new();
		mixer.set_listener(Listener {
			position: Vec3::ZERO,
			forward: Vec3::FORWARD,
			up: Vec3::UP,
		});

		// 1. Source on the right: (5.0, 0.0, 0.0)
		let right_source = Arc::new(Mutex::new(AudioSource::new()));
		{
			let mut s = right_source.lock().unwrap();
			s.bind_clip(Arc::clone(&clip));
			s.set_position(Vec3::new_with(5.0, 0.0, 0.0));
			s.min_distance = 10.0; // no attenuation
			s.play();
		}
		mixer.add_source(Arc::clone(&right_source));

		let mut buffer = vec![0.0f32; 20]; // 10 frames stereo
		mixer.process(&mut buffer, 10);

		// right = forward x up = (0,0,-1) x (0,1,0) = (1,0,0).
		// rel_pos = (5,0,0). pan = dot(dir, right) = 1.0.
		// vol_l = 1.0 - 1.0 = 0.0.
		// vol_r = 1.0 + 1.0 = 1.0 (clamped).
		for i in 0..10 {
			assert_eq!(buffer[i * 2 + 0], 0.0); // Left channel is silent
			assert!(buffer[i * 2 + 1] > 0.9);  // Right channel is loud
		}

		// 2. Source beyond max distance
		mixer.remove_source(&right_source);
		let far_source = Arc::new(Mutex::new(AudioSource::new()));
		{
			let mut s = far_source.lock().unwrap();
			s.bind_clip(Arc::clone(&clip));
			s.set_position(Vec3::new_with(0.0, 0.0, -500.0));
			s.min_distance = 1.0;
			s.max_distance = 50.0;
			s.play();
		}
		mixer.add_source(far_source);

		let mut silent_buffer = vec![0.0f32; 20];
		mixer.process(&mut silent_buffer, 10);
		for sample in silent_buffer {
			assert_eq!(sample, 0.0);
		}
	}

	#[test]
	fn test_mixer_stopped_sources_cleanup() {
		let pcm = vec![1.0; 8];
		let clip = Arc::new(AudioClip::new(44100, 2, pcm));

		let mut mixer = Mixer::new();
		let s1 = Arc::new(Mutex::new(AudioSource::new()));
		let s2 = Arc::new(Mutex::new(AudioSource::new()));

		s1.lock().unwrap().bind_clip(Arc::clone(&clip));
		s1.lock().unwrap().play();

		s2.lock().unwrap().bind_clip(Arc::clone(&clip));
		s2.lock().unwrap().stop();

		mixer.add_source(Arc::clone(&s1));
		mixer.add_source(Arc::clone(&s2));
		assert_eq!(mixer.source_count(), 2);

		mixer.remove_stopped_sources();
		assert_eq!(mixer.source_count(), 1);
	}
}

