/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_math::Vec3;
use crate::body::BodyHandle;
use crate::collider::ColliderHandle;


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ray {
	pub origin: Vec3<f32>,
	pub direction: Vec3<f32>,
	pub max_distance: f32,
}

impl Ray {
	pub fn new(origin: Vec3<f32>, direction: Vec3<f32>, max_distance: f32) -> Self {
		Self {
			origin,
			direction: direction.normalize(),
			max_distance,
		}
	}

	#[inline]
	pub fn point_at(&self, t: f32) -> Vec3<f32> {
		self.origin + self.direction * t
	}
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RaycastHit {
	pub body: BodyHandle,
	pub collider: ColliderHandle,
	pub point: Vec3<f32>,
	pub normal: Vec3<f32>,
	pub distance: f32,
}
