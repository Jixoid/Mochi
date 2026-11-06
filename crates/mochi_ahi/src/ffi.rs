/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ffi::c_char;


#[repr(C)]
pub struct MochiAudioDeviceOpaque { _private: [u8; 0] }

pub type MochiAudioDeviceHandle = *mut MochiAudioDeviceOpaque;

pub type MochiAudioCallback = Option<unsafe extern "C" fn(user_data: *mut std::ffi::c_void, output: *mut f32, frame_count: u32)>;


unsafe extern "C" {
	pub unsafe fn mochi_ahi_device_new(app_name: *const c_char, major: u16, minor: u16, patch: u16, build: u16) -> MochiAudioDeviceHandle;
	pub unsafe fn mochi_ahi_device_dis(device: MochiAudioDeviceHandle);

	pub unsafe fn mochi_ahi_device_set_callback(device: MochiAudioDeviceHandle, callback: MochiAudioCallback, user_data: *mut std::ffi::c_void);
	pub unsafe fn mochi_ahi_device_start(device: MochiAudioDeviceHandle) -> bool;
	pub unsafe fn mochi_ahi_device_stop(device: MochiAudioDeviceHandle);	
	pub unsafe fn mochi_ahi_device_is_playing(device: MochiAudioDeviceHandle) -> bool;
}
