/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{Deg, Float, Quat, Rad};


#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Euler<T: Float = f32> {
	pub pitch: Rad<T>,
	pub yaw: Rad<T>,
	pub roll: Rad<T>,
}


impl<T: Float> Euler<T> {
	pub const ZERO: Self = Self {
		pitch: Rad::ZERO,
		yaw: Rad::ZERO,
		roll: Rad::ZERO,
	};

	pub const fn new(pitch: Rad<T>, yaw: Rad<T>, roll: Rad<T>) -> Self {
		Self { pitch, yaw, roll }
	}

	pub fn from_deg(pitch: Deg<T>, yaw: Deg<T>, roll: Deg<T>) -> Self {
		Self {
			pitch: pitch.to_radians(),
			yaw: yaw.to_radians(),
			roll: roll.to_radians(),
		}
	}

	pub fn to_quat(&self) -> Quat<T> {
		Quat::from(*self)
	}
}


impl<T: Float> From<Quat<T>> for Euler<T> {
	fn from(value: Quat<T>) -> Self {
		let q = value.normalize();

		// YXZ order extraction
		let sin_p = -T::TWO * (q.y * q.z - q.w * q.x);
		let pitch = if sin_p.abs() >= T::ONE {
			let half_pi = T::PI * T::HALF;
			if sin_p > T::ZERO {
				Rad::new(half_pi)
			} else {
				Rad::new(-half_pi)
			}
		} else {
			Rad::asin(sin_p)
		};

		let yaw = Rad::atan2(
			T::TWO * (q.x * q.z + q.w * q.y),
			T::ONE - T::TWO * (q.x * q.x + q.y * q.y),
		);

		let roll = Rad::atan2(
			T::TWO * (q.x * q.y + q.w * q.z),
			T::ONE - T::TWO * (q.x * q.x + q.z * q.z),
		);

		Self { pitch, yaw, roll }
	}
}

impl<T: Float> From<(Rad<T>, Rad<T>, Rad<T>)> for Euler<T> {
	fn from((pitch, yaw, roll): (Rad<T>, Rad<T>, Rad<T>)) -> Self {
		Self::new(pitch, yaw, roll)
	}
}

impl<T: Float> From<(Deg<T>, Deg<T>, Deg<T>)> for Euler<T> {
	fn from((pitch, yaw, roll): (Deg<T>, Deg<T>, Deg<T>)) -> Self {
		Self::from_deg(pitch, yaw, roll)
	}
}
