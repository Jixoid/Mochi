/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_ahi::error::AhiResult;
use mochi_ahi::traits::AhiBackend;

use crate::device::CpalAudioDevice;


pub struct CpalAudioBackend;

impl AhiBackend for CpalAudioBackend {
	type Device = CpalAudioDevice;

	fn create_device(
		app_name: &str,
		major: u16,
		minor: u16,
		patch: u16,
		build: u16,
	) -> AhiResult<Self::Device> {
		CpalAudioDevice::new(app_name, major, minor, patch, build)
	}
}
