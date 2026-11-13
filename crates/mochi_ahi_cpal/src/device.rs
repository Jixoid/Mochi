/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, Mutex};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use symphonia::core::audio::{Audio, GenericAudioBufferRef};
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use mochi_ahi::data::{AudioCallback, AudioData};
use mochi_ahi::error::{AhiError, AhiResult};
use mochi_ahi::traits::AhiDevice;
use mochi_except::*;


pub struct CpalAudioDevice {
	device: Option<cpal::Device>,
	stream: Option<Stream>,
	callback: Arc<Mutex<Option<AudioCallback>>>,
	is_playing: Arc<Mutex<bool>>,
	sample_rate: u32,
	channels: u16,
}

unsafe impl Send for CpalAudioDevice {}
unsafe impl Sync for CpalAudioDevice {}

impl CpalAudioDevice {
	pub fn new(
		app_name: &str,
		_major: u16,
		_minor: u16,
		_patch: u16,
		_build: u16,
	) -> AhiResult<Self> {
		let host = cpal::default_host();
		let device = host.default_output_device();

		if let Some(dev) = &device {
			let dev_name = dev
				.description()
				.map(|d| d.name().to_string())
				.unwrap_or_else(|_| "Default".to_string());
			mochi_disp!("CPAL audio device initialized for {app_name}: {dev_name}");
		} else {
			mochi_warn!("No audio output device available for {app_name}");
		}

		Ok(Self {
			device,
			stream: None,
			callback: Arc::new(Mutex::new(None)),
			is_playing: Arc::new(Mutex::new(false)),
			sample_rate: 44100,
			channels: 2,
		})
	}
}


impl AhiDevice for CpalAudioDevice {
	fn sample_rate(&self) -> u32 {
		self.sample_rate
	}

	fn channels(&self) -> u16 {
		self.channels
	}

	fn set_callback(&mut self, callback: AudioCallback) {
		*self.callback.lock().unwrap() = Some(callback);

		let Some(device) = &self.device else {
			return;
		};

		let default_config = match device.default_output_config() {
			Ok(cfg) => cfg,
			Err(err) => {
				mochi_warn!("Could not query default audio config: {err}");
				return;
			}
		};

		let fmt = default_config.sample_format();
		let config = StreamConfig {
			channels: 2,
			sample_rate: 44100,
			buffer_size: cpal::BufferSize::Default,
		};

		let callback_clone = Arc::clone(&self.callback);
		let is_playing_clone = Arc::clone(&self.is_playing);
		let err_fn = |err| mochi_error!("Stream error: {err}");

		let stream_res = match fmt {
			SampleFormat::F32 => {
				let cfg = config;
				device.build_output_stream(
					cfg,
					move |data: &mut [f32], _| {
						if *is_playing_clone.lock().unwrap() {
							let frame_count = (data.len() / 2) as u32;
							if let Ok(mut cb) = callback_clone.lock()
								&& let Some(f) = cb.as_mut()
							{
								f(data, frame_count);
								return;
							}
						}
						data.fill(0.0);
					},
					err_fn,
					None,
				)
			}
			SampleFormat::I16 => {
				let cfg = config;
				let mut temp_buf = Vec::new();
				device.build_output_stream(
					cfg,
					move |data: &mut [i16], _| {
						let len = data.len();
						temp_buf.resize(len, 0.0f32);
						if *is_playing_clone.lock().unwrap() {
							let frame_count = (len / 2) as u32;
							if let Ok(mut cb) = callback_clone.lock()
								&& let Some(f) = cb.as_mut()
							{
								f(&mut temp_buf, frame_count);
								for (out, &sample) in data.iter_mut().zip(&temp_buf) {
									*out = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
								}
								return;
							}
						}
						data.fill(0);
					},
					err_fn,
					None,
				)
			}
			SampleFormat::U16 => {
				let cfg = config;
				let mut temp_buf = Vec::new();
				device.build_output_stream(
					cfg,
					move |data: &mut [u16], _| {
						let len = data.len();
						temp_buf.resize(len, 0.0f32);
						if *is_playing_clone.lock().unwrap() {
							let frame_count = (len / 2) as u32;
							if let Ok(mut cb) = callback_clone.lock()
								&& let Some(f) = cb.as_mut()
							{
								f(&mut temp_buf, frame_count);
								for (out, &sample) in data.iter_mut().zip(&temp_buf) {
									*out = ((sample.clamp(-1.0, 1.0) * 32767.0) + 32768.0) as u16;
								}
								return;
							}
						}
						data.fill(32768);
					},
					err_fn,
					None,
				)
			}
			_ => {
				mochi_error!("Unsupported audio sample format: {fmt:?}");
				return;
			}
		};

		match stream_res {
			Ok(stream) => {
				self.sample_rate = config.sample_rate;
				self.channels = config.channels;
				self.stream = Some(stream);
			}
			Err(err) => {
				mochi_error!("Failed to build audio output stream: {err}");
			}
		}
	}

	fn start(&mut self) -> bool {
		if let Some(stream) = &self.stream
			&& stream.play().is_ok()
		{
			*self.is_playing.lock().unwrap() = true;
			return true;
		}
		false
	}

	fn stop(&mut self) {
		*self.is_playing.lock().unwrap() = false;
		if let Some(stream) = &self.stream {
			let _ = stream.pause();
		}
	}

	fn is_playing(&self) -> bool {
		*self.is_playing.lock().unwrap()
	}

	fn decode_file(&self, path: &str) -> AhiResult<AudioData> {
		let file = File::open(path)
			.map_err(|e| AhiError::IoError(format!("Failed to open audio file {path}: {e}")))?;
		decode_media_source(file, Some(path))
	}

	fn decode_memory(&self, bytes: &[u8]) -> AhiResult<AudioData> {
		decode_media_source(Cursor::new(bytes.to_vec()), None)
	}
}


impl Drop for CpalAudioDevice {
	fn drop(&mut self) {
		self.stop();
		self.stream = None;
	}
}


fn decode_media_source<R: MediaSource + 'static>(
	source: R,
	hint_path: Option<&str>,
) -> AhiResult<AudioData> {
	let mut hint = Hint::new();
	if let Some(path) = hint_path
		&& let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str())
	{
		hint.with_extension(ext);
	}

	let mss = MediaSourceStream::new(Box::new(source), Default::default());
	let format_opts = FormatOptions::default();
	let metadata_opts = MetadataOptions::default();

	let mut format = symphonia::default::get_probe()
		.probe(&hint, mss, format_opts, metadata_opts)
		.map_err(|e| AhiError::DecodeError(format!("Failed to probe audio format: {e}")))?;

	let track = format
		.default_track(symphonia::core::formats::TrackType::Audio)
		.or_else(|| {
			format
				.tracks()
				.iter()
				.find(|t| t.codec_params.as_ref().is_some_and(|p| p.is_audio()))
		})
		.ok_or_else(|| AhiError::DecodeError("No supported audio tracks found".to_string()))?;

	let track_id = track.id;
	let codec_params = track
		.codec_params
		.as_ref()
		.and_then(|p| p.audio())
		.ok_or_else(|| {
			AhiError::DecodeError("Track does not contain audio codec parameters".to_string())
		})?;

	let mut decoder = symphonia::default::get_codecs()
		.make_audio_decoder(codec_params, &AudioDecoderOptions::default())
		.map_err(|e| AhiError::DecodeError(format!("Failed to create audio decoder: {e}")))?;

	let mut sample_rate = 0;
	let mut channels = 0;
	let mut pcm_data = Vec::new();

	while let Ok(Some(packet)) = format.next_packet() {
		if packet.track_id != track_id {
			continue;
		}

		match decoder.decode(&packet) {
			Ok(decoded) => match decoded {
				GenericAudioBufferRef::F32(buf) => {
					sample_rate = buf.spec().rate();
					channels = buf.spec().channels().count() as u8;
					for frame in 0..buf.frames() {
						for ch in 0..channels as usize {
							if let Some(plane) = buf.plane(ch) {
								pcm_data.push(plane[frame]);
							}
						}
					}
				}
				GenericAudioBufferRef::S16(buf) => {
					sample_rate = buf.spec().rate();
					channels = buf.spec().channels().count() as u8;
					for frame in 0..buf.frames() {
						for ch in 0..channels as usize {
							if let Some(plane) = buf.plane(ch) {
								pcm_data.push(plane[frame] as f32 / 32768.0);
							}
						}
					}
				}
				GenericAudioBufferRef::U8(buf) => {
					sample_rate = buf.spec().rate();
					channels = buf.spec().channels().count() as u8;
					for frame in 0..buf.frames() {
						for ch in 0..channels as usize {
							if let Some(plane) = buf.plane(ch) {
								pcm_data.push((plane[frame] as f32 - 128.0) / 128.0);
							}
						}
					}
				}
				GenericAudioBufferRef::S32(buf) => {
					sample_rate = buf.spec().rate();
					channels = buf.spec().channels().count() as u8;
					for frame in 0..buf.frames() {
						for ch in 0..channels as usize {
							if let Some(plane) = buf.plane(ch) {
								pcm_data.push(plane[frame] as f32 / 2147483648.0);
							}
						}
					}
				}
				GenericAudioBufferRef::F64(buf) => {
					sample_rate = buf.spec().rate();
					channels = buf.spec().channels().count() as u8;
					for frame in 0..buf.frames() {
						for ch in 0..channels as usize {
							if let Some(plane) = buf.plane(ch) {
								pcm_data.push(plane[frame] as f32);
							}
						}
					}
				}
				_ => {}
			},
			Err(symphonia::core::errors::Error::IoError(err)) => {
				if err.kind() == std::io::ErrorKind::UnexpectedEof {
					break;
				}
				return Err(AhiError::IoError(format!("Audio decode IO error: {err}")));
			}
			Err(symphonia::core::errors::Error::DecodeError(err)) => {
				mochi_warn!("Corrupted audio packet ignored: {err}");
			}
			Err(err) => return Err(AhiError::DecodeError(format!("Audio decode error: {err}"))),
		}
	}

	Ok(AudioData::new(sample_rate, channels, pcm_data))
}
