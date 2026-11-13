/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::data::{AudioCallback, AudioData};
use crate::error::AhiResult;


/// Abstract AHI backend trait implemented by audio drivers (e.g. CPAL)
pub trait AhiBackend: 'static + Sized + Send + Sync {
	type Device: AhiDevice;

	fn create_device(
		app_name: &str,
		major: u16,
		minor: u16,
		patch: u16,
		build: u16,
	) -> AhiResult<Self::Device>;
}


/// Abstract audio hardware device interface
pub trait AhiDevice: Send + Sync + 'static {
	fn sample_rate(&self) -> u32;
	fn channels(&self) -> u16;
	fn set_callback(&mut self, callback: AudioCallback);
	fn start(&mut self) -> bool;
	fn stop(&mut self);
	fn is_playing(&self) -> bool;
	fn decode_file(&self, path: &str) -> AhiResult<AudioData>;
	fn decode_memory(&self, bytes: &[u8]) -> AhiResult<AudioData>;
}
