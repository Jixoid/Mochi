/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_audio::AudioClip;
use mochi_core::Engine;
use mochi_except::*;


fn main() {
	mochi_except::start();
	let mut engine = Engine::from_config("mochi.ron");

	// Play a procedural startup chime to verify audio subsystem
	if let Some(audio) = engine.audio_system() {
		let startup_chime = std::sync::Arc::new(AudioClip::sine_wave(523.25, 0.35, 44100));
		audio.play_one_shot(startup_chime, 0.15);
	}

	engine.set_idle(|_dt| {
		// Frame update
	});

	if let Err(err) = engine.run() {
		mochi_error!("Runtime error: {err}");
	}
}
