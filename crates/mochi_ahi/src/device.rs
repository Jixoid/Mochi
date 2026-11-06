/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ffi::CString;
use crate::ffi::*;


unsafe extern "C" fn audio_callback_trampoline(user_data: *mut std::ffi::c_void, output: *mut f32, frame_count: u32) {
	if user_data.is_null() || output.is_null() {
		return;
	}
	unsafe {
		let callback = &mut *(user_data as *mut Box<dyn FnMut(&mut [f32], u32) + Send>);
		let slice = std::slice::from_raw_parts_mut(output, (frame_count * 2) as usize);
		callback(slice, frame_count);
	}
}


pub struct Device {
	handle: MochiAudioDeviceHandle,
	callback_box: Option<*mut Box<dyn FnMut(&mut [f32], u32) + Send>>,
}

unsafe impl Send for Device {}

impl Device {
	pub fn new(app_name: &str, major: u16, minor: u16, patch: u16, build: u16) -> Self {
		let c_str = CString::new(app_name).expect("C string failed");
		let handle = unsafe { mochi_ahi_device_new(c_str.as_ptr(), major, minor, patch, build) };

		assert!(!handle.is_null(), "AHI driver failed to create Device");

		Self {
			handle,
			callback_box: None,
		}
	}

	pub fn handle(&self) -> MochiAudioDeviceHandle {
		self.handle
	}

	
	pub fn set_callback<F: FnMut(&mut [f32], u32) + Send + 'static>(&mut self, callback: F) {
		if let Some(prev) = self.callback_box.take() {
			unsafe {
				drop(Box::from_raw(prev));
			}
		}

		let boxed: Box<dyn FnMut(&mut [f32], u32) + Send> = Box::new(callback);
		let heap_box = Box::new(boxed);
		let raw_ptr = Box::into_raw(heap_box);

		unsafe {
			mochi_ahi_device_set_callback(
				self.handle,
				Some(audio_callback_trampoline),
				raw_ptr as *mut std::ffi::c_void,
			);
		}

		self.callback_box = Some(raw_ptr);
	}

	pub fn start(&mut self) -> bool {
		unsafe { mochi_ahi_device_start(self.handle) }
	}

	pub fn stop(&mut self) {
		unsafe { mochi_ahi_device_stop(self.handle) }
	}

	pub fn is_playing(&self) -> bool {
		unsafe { mochi_ahi_device_is_playing(self.handle) }
	}

	pub fn decode_file(&self, path: &str) -> Result<crate::data::AudioData, String> {
		let c_path = CString::new(path).map_err(|e| e.to_string())?;
		let mut audio_data = MochiAudioData {
			sample_rate: 0,
			channels: 0,
			frame_count: 0,
			pcm_data: std::ptr::null_mut(),
		};

		let success = unsafe {
			mochi_ahi_device_decode_file(
				self.handle,
				c_path.as_ptr(),
				&mut audio_data,
			)
		};

		if !success || audio_data.pcm_data.is_null() {
			return Err(format!("Failed to decode audio file: {path}"));
		}

		let total_samples = (audio_data.frame_count * audio_data.channels as u64) as usize;
		let pcm_slice = unsafe { std::slice::from_raw_parts(audio_data.pcm_data, total_samples) };
		let pcm_vec = pcm_slice.to_vec();

		unsafe {
			mochi_ahi_free_audio_data(&mut audio_data);
		}

		Ok(crate::data::AudioData {
			sample_rate: audio_data.sample_rate,
			channels: audio_data.channels,
			pcm_data: pcm_vec,
		})
	}

	pub fn decode_memory(&self, bytes: &[u8]) -> Result<crate::data::AudioData, String> {
		let mut audio_data = MochiAudioData {
			sample_rate: 0,
			channels: 0,
			frame_count: 0,
			pcm_data: std::ptr::null_mut(),
		};

		let success = unsafe {
			mochi_ahi_device_decode_memory(
				self.handle,
				bytes.as_ptr(),
				bytes.len(),
				&mut audio_data,
			)
		};

		if !success || audio_data.pcm_data.is_null() {
			return Err("Failed to decode audio from memory".to_string());
		}

		let total_samples = (audio_data.frame_count * audio_data.channels as u64) as usize;
		let pcm_slice = unsafe { std::slice::from_raw_parts(audio_data.pcm_data, total_samples) };
		let pcm_vec = pcm_slice.to_vec();

		unsafe {
			mochi_ahi_free_audio_data(&mut audio_data);
		}

		Ok(crate::data::AudioData {
			sample_rate: audio_data.sample_rate,
			channels: audio_data.channels,
			pcm_data: pcm_vec,
		})
	}
}

impl Drop for Device {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_ahi_device_stop(self.handle);
				mochi_ahi_device_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}

		if let Some(ptr) = self.callback_box.take() {
			unsafe {
				drop(Box::from_raw(ptr));
			}
		}
	}
}
