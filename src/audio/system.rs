/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};
use mochi_ahi::Device;
use crate::audio::{AudioClip, AudioSource, Listener, Mixer};


pub struct AudioSystem {
	device: Device,
	mixer: Arc<Mutex<Mixer>>,
}

impl AudioSystem {
	pub fn new(app_name: &str, major: u16, minor: u16, patch: u16, build: u16) -> Self {
		let mut device = Device::new(app_name, major, minor, patch, build);
		let mixer = Arc::new(Mutex::new(Mixer::new()));

		let mixer_cb = Arc::clone(&mixer);
		device.set_callback(move |output, frame_count| {
			if let Ok(mut m) = mixer_cb.lock() {
				m.process(output, frame_count);
			}
		});

		let started = device.start();
		if started {
			println!("[AudioSystem] Audio device started successfully (44100Hz, Stereo Float).");
		} else {
			eprintln!("[AudioSystem] Failed to start audio device!");
		}

		Self { device, mixer }
	}

	pub fn device(&self) -> &Device {
		&self.device
	}

	pub fn mixer(&self) -> &Arc<Mutex<Mixer>> {
		&self.mixer
	}

	pub fn set_master_volume(&self, volume: f32) {
		if let Ok(mut mixer) = self.mixer.lock() {
			mixer.set_master_volume(volume);
		}
	}

	pub fn master_volume(&self) -> f32 {
		self.mixer.lock().map_or(1.0, |m| m.master_volume())
	}

	pub fn set_listener(&self, listener: Listener) {
		if let Ok(mut mixer) = self.mixer.lock() {
			mixer.set_listener(listener);
		}
	}

	pub fn set_listener_transform(
		&self,
		position: impl Into<mochi_math::Vec3<f32>>,
		forward: impl Into<mochi_math::Vec3<f32>>,
		up: impl Into<mochi_math::Vec3<f32>>,
	) {
		if let Ok(mut mixer) = self.mixer.lock() {
			mixer.set_listener(Listener {
				position: position.into(),
				forward: forward.into(),
				up: up.into(),
			});
		}
	}

	pub fn create_source(&self, clip: Option<Arc<AudioClip>>) -> Arc<Mutex<AudioSource>> {
		let mut source = AudioSource::new();
		if let Some(c) = clip {
			source.bind_clip(c);
		}
		let source_arc = Arc::new(Mutex::new(source));

		if let Ok(mut mixer) = self.mixer.lock() {
			mixer.add_source(Arc::clone(&source_arc));
		}

		source_arc
	}

	pub fn play_one_shot(&self, clip: Arc<AudioClip>, volume: f32) -> Arc<Mutex<AudioSource>> {
		let mut source = AudioSource::new();
		source.bind_clip(clip);
		source.set_volume(volume);
		source.play();

		let source_arc = Arc::new(Mutex::new(source));

		if let Ok(mut mixer) = self.mixer.lock() {
			mixer.add_source(Arc::clone(&source_arc));
		}

		source_arc
	}

	pub fn play_one_shot_3d(
		&self,
		clip: Arc<AudioClip>,
		position: impl Into<mochi_math::Vec3<f32>>,
		volume: f32,
	) -> Arc<Mutex<AudioSource>> {
		let mut source = AudioSource::new();
		source.bind_clip(clip);
		source.set_position(position.into());
		source.set_volume(volume);
		source.play();

		let source_arc = Arc::new(Mutex::new(source));

		if let Ok(mut mixer) = self.mixer.lock() {
			mixer.add_source(Arc::clone(&source_arc));
		}

		source_arc
	}

	pub fn update(&self) {
		if let Ok(mut mixer) = self.mixer.lock() {
			mixer.remove_stopped_sources();
		}
	}
}
