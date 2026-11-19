/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_math::{Quat, Vec3};
use crate::body::BodyHandle;


#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColliderHandle(pub u32);

impl ColliderHandle {
	pub const INVALID: Self = Self(u32::MAX);

	#[inline]
	pub fn is_valid(self) -> bool {
		self != Self::INVALID
	}
}

impl Default for ColliderHandle {
	fn default() -> Self {
		Self::INVALID
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PhysicsMaterial {
	/// Surface Coulomb friction coefficient in range [0.0, 1.0].
	pub friction: f32,
	/// Coefficient of restitution / bounciness in range [0.0, 1.0].
	pub restitution: f32,
	/// Volumetric material density in kg/m^3 (default 1000.0).
	pub density: f32,
}

impl Default for PhysicsMaterial {
	fn default() -> Self {
		Self {
			friction: 0.4,
			restitution: 0.0,
			density: 1000.0,
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColliderShape {
	/// Sphere primitive defined by radius.
	Sphere { radius: f32 },
	/// Axis-aligned box defined by symmetric half extents vector.
	Box { half_extents: Vec3<f32> },
	/// Capsule oriented along Y-axis with hemispherical caps and cylinder height.
	Capsule { radius: f32, height: f32 },
	/// Cylinder oriented along Y-axis defined by radius and height.
	Cylinder { radius: f32, height: f32 },
	/// Convex polygon cloud of vertices forming a convex hull.
	ConvexHull { vertices: Vec<Vec3<f32>> },
	/// Static triangle mesh with vertices and indexed triangle faces.
	TriMesh {
		vertices: Vec<Vec3<f32>>,
		indices: Vec<u32>,
	},
}

impl ColliderShape {
	pub fn sphere(radius: f32) -> Self {
		Self::Sphere { radius }
	}

	pub fn box_shape(half_x: f32, half_y: f32, half_z: f32) -> Self {
		Self::Box {
			half_extents: Vec3::new(half_x, half_y, half_z),
		}
	}

	pub fn capsule(radius: f32, height: f32) -> Self {
		Self::Capsule { radius, height }
	}
}


#[derive(Clone, Debug, PartialEq)]
pub struct ColliderDef {
	pub shape: ColliderShape,
	pub material: PhysicsMaterial,
	pub offset_position: Vec3<f32>,
	pub offset_rotation: Quat<f32>,
	pub is_trigger: bool,
}

impl Default for ColliderDef {
	fn default() -> Self {
		Self {
			shape: ColliderShape::box_shape(0.5, 0.5, 0.5),
			material: PhysicsMaterial::default(),
			offset_position: Vec3::ZERO,
			offset_rotation: Quat::IDENTITY,
			is_trigger: false,
		}
	}
}

impl ColliderDef {
	pub fn new(shape: ColliderShape) -> Self {
		Self {
			shape,
			..Default::default()
		}
	}

	pub fn with_material(mut self, material: PhysicsMaterial) -> Self {
		self.material = material;
		self
	}

	pub fn with_offset(mut self, pos: Vec3<f32>, rot: Quat<f32>) -> Self {
		self.offset_position = pos;
		self.offset_rotation = rot;
		self
	}

	pub fn as_trigger(mut self, is_trigger: bool) -> Self {
		self.is_trigger = is_trigger;
		self
	}
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ContactPoint {
	pub position: Vec3<f32>,
	pub normal: Vec3<f32>,
	pub penetration_depth: f32,
	pub normal_impulse: f32,
	pub tangent_impulse: f32,
}


#[derive(Clone, Debug, PartialEq)]
pub struct ContactManifold {
	pub body_a: BodyHandle,
	pub body_b: BodyHandle,
	pub points: Vec<ContactPoint>,
}
