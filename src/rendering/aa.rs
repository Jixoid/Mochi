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
	Smaa1x = 5,
	SmaaT2x = 6,
	SmaaS2x = 7,
	Smaa4x = 8,
	Taa = 9,
	FsrNativeAa = 10,
	FsrQuality = 11,
	FsrBalanced = 12,
	FsrPerformance = 13,
	FsrUltraPerformance = 14,
}

impl AntiAliasingMode {
	pub fn from_i32(val: i32) -> Self {
		match val {
			1 => Self::Msaa2x,
			2 => Self::Msaa4x,
			3 => Self::Msaa8x,
			4 => Self::Fxaa,
			5 => Self::Smaa1x,
			6 => Self::SmaaT2x,
			7 => Self::SmaaS2x,
			8 => Self::Smaa4x,
			9 => Self::Taa,
			10 => Self::FsrNativeAa,
			11 => Self::FsrQuality,
			12 => Self::FsrBalanced,
			13 => Self::FsrPerformance,
			14 => Self::FsrUltraPerformance,
			_ => Self::Off,
		}
	}

	pub fn to_i32(self) -> i32 {
		self as i32
	}

	pub fn required_msaa(self) -> MsaaSamples {
		match self {
			Self::Msaa2x | Self::SmaaS2x | Self::Smaa4x => MsaaSamples::X2,
			Self::Msaa4x => MsaaSamples::X4,
			Self::Msaa8x => MsaaSamples::X8,
			_ => MsaaSamples::Off,
		}
	}

	pub fn msaa_samples(self) -> MsaaSamples {
		self.required_msaa()
	}

	pub fn is_msaa_only(self) -> bool {
		matches!(self, Self::Msaa2x | Self::Msaa4x | Self::Msaa8x)
	}

	pub fn is_post_process(self) -> bool {
		!matches!(self, Self::Off | Self::Msaa2x | Self::Msaa4x | Self::Msaa8x)
	}

	pub fn is_fxaa(self) -> bool {
		matches!(self, Self::Fxaa)
	}

	pub fn is_smaa(self) -> bool {
		matches!(
			self,
			Self::Smaa1x | Self::SmaaT2x | Self::SmaaS2x | Self::Smaa4x
		)
	}

	pub fn is_taa(self) -> bool {
		matches!(self, Self::Taa)
	}

	pub fn is_fsr(self) -> bool {
		matches!(
			self,
			Self::FsrNativeAa
				| Self::FsrQuality
				| Self::FsrBalanced
				| Self::FsrPerformance
				| Self::FsrUltraPerformance
		)
	}

	pub fn needs_velocity(self) -> bool {
		matches!(
			self,
			Self::SmaaT2x
				| Self::Smaa4x
				| Self::Taa
				| Self::FsrNativeAa
				| Self::FsrQuality
				| Self::FsrBalanced
				| Self::FsrPerformance
				| Self::FsrUltraPerformance
		)
	}

	pub fn needs_history(self) -> bool {
		matches!(
			self,
			Self::SmaaT2x
				| Self::Smaa4x
				| Self::Taa
				| Self::FsrNativeAa
				| Self::FsrQuality
				| Self::FsrBalanced
				| Self::FsrPerformance
				| Self::FsrUltraPerformance
		)
	}

	pub fn needs_camera_jitter(self) -> bool {
		matches!(
			self,
			Self::SmaaT2x
				| Self::Smaa4x
				| Self::Taa
				| Self::FsrNativeAa
				| Self::FsrQuality
				| Self::FsrBalanced
				| Self::FsrPerformance
				| Self::FsrUltraPerformance
		)
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::Off => "Off",
			Self::Msaa2x => "MSAA 2x",
			Self::Msaa4x => "MSAA 4x",
			Self::Msaa8x => "MSAA 8x",
			Self::Fxaa => "FXAA",
			Self::Smaa1x => "SMAA 1x",
			Self::SmaaT2x => "SMAA T2x",
			Self::SmaaS2x => "SMAA S2x",
			Self::Smaa4x => "SMAA 4x",
			Self::Taa => "TAA",
			Self::FsrNativeAa => "FSR Native AA",
			Self::FsrQuality => "FSR Quality",
			Self::FsrBalanced => "FSR Balanced",
			Self::FsrPerformance => "FSR Performance",
			Self::FsrUltraPerformance => "FSR Ultra Performance",
		}
	}
}

pub fn halton(index: u32, base: u32) -> f32 {
	let mut f = 1.0;
	let mut r = 0.0;
	let mut i = index;
	let base_f = base as f32;

	while i > 0 {
		f /= base_f;
		r += f * (i % base) as f32;
		i /= base;
	}
	r
}

pub fn get_subpixel_jitter(mode: AntiAliasingMode, frame_index: u64) -> (f32, f32) {
	match mode {
		AntiAliasingMode::SmaaT2x | AntiAliasingMode::Smaa4x => {
			if frame_index % 2 == 0 {
				(0.25, -0.25)
			} else {
				(-0.25, 0.25)
			}
		}
		AntiAliasingMode::Taa
		| AntiAliasingMode::FsrNativeAa
		| AntiAliasingMode::FsrQuality
		| AntiAliasingMode::FsrBalanced
		| AntiAliasingMode::FsrPerformance
		| AntiAliasingMode::FsrUltraPerformance => {
			let idx = ((frame_index % 16) + 1) as u32;
			(halton(idx, 2) - 0.5, halton(idx, 3) - 0.5)
		}
		_ => (0.0, 0.0),
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_halton_sequence_bounds() {
		for i in 1..=32 {
			let h2 = halton(i, 2);
			let h3 = halton(i, 3);
			assert!(h2 > 0.0 && h2 < 1.0);
			assert!(h3 > 0.0 && h3 < 1.0);
		}
	}

	#[test]
	fn test_subpixel_jitter_bounds() {
		for frame in 0..64 {
			let (jx, jy) = get_subpixel_jitter(AntiAliasingMode::Taa, frame);
			assert!((-0.5..=0.5).contains(&jx));
			assert!((-0.5..=0.5).contains(&jy));
		}
	}

	#[test]
	fn test_feature_requirements() {
		assert!(!AntiAliasingMode::Off.needs_velocity());
		assert!(!AntiAliasingMode::Msaa4x.needs_velocity());
		assert!(!AntiAliasingMode::Fxaa.needs_velocity());
		assert!(!AntiAliasingMode::Smaa1x.needs_velocity());

		assert!(AntiAliasingMode::SmaaT2x.needs_velocity());
		assert!(AntiAliasingMode::Smaa4x.needs_velocity());
		assert!(AntiAliasingMode::Taa.needs_velocity());
		assert!(AntiAliasingMode::FsrNativeAa.needs_velocity());

		assert_eq!(AntiAliasingMode::Msaa2x.required_msaa(), MsaaSamples::X2);
		assert_eq!(AntiAliasingMode::Msaa4x.required_msaa(), MsaaSamples::X4);
		assert_eq!(AntiAliasingMode::Msaa8x.required_msaa(), MsaaSamples::X8);
		assert_eq!(AntiAliasingMode::SmaaT2x.required_msaa(), MsaaSamples::Off);
		assert_eq!(AntiAliasingMode::SmaaS2x.required_msaa(), MsaaSamples::X2);
		assert_eq!(AntiAliasingMode::Smaa4x.required_msaa(), MsaaSamples::X2);
		assert_eq!(AntiAliasingMode::Taa.required_msaa(), MsaaSamples::Off);
	}
}
