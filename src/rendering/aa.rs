/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::rhi::MsaaSamples;


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum AntiAliasingMode {
	#[default]
	Off = 0,
	Msaa2x = 1,
	Msaa4x = 2,
	Msaa8x = 3,
	Fxaa = 4,
}

impl AntiAliasingMode {
	pub fn from_i32(val: i32) -> Self {
		match val {
			1 => Self::Msaa2x,
			2 => Self::Msaa4x,
			3 => Self::Msaa8x,
			4 => Self::Fxaa,
			_ => Self::Off,
		}
	}

	pub fn to_i32(self) -> i32 {
		self as i32
	}

	pub fn msaa_samples(self) -> MsaaSamples {
		match self {
			Self::Msaa2x => MsaaSamples::X2,
			Self::Msaa4x => MsaaSamples::X4,
			Self::Msaa8x => MsaaSamples::X8,
			_ => MsaaSamples::Off,
		}
	}

	pub fn is_fxaa(self) -> bool {
		matches!(self, Self::Fxaa)
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::Off => "Off",
			Self::Msaa2x => "MSAA 2x",
			Self::Msaa4x => "MSAA 4x",
			Self::Msaa8x => "MSAA 8x",
			Self::Fxaa => "FXAA",
		}
	}
}
