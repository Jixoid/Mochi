/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/

//! # Mochi Physics Hardware/Backend Interface (PHI)
//!
//! Abstract, backend-agnostic 3D physics interfaces, components, and math abstractions for the Mochi Engine.
//! Designed to support multiple pluggable backends such as Baryon (C++/Rust) or Rapier3D.

pub mod body;
pub mod collider;
pub mod error;
pub mod joint;
pub mod ray;
pub mod traits;
pub mod world;

pub use body::*;
pub use collider::*;
pub use error::*;
pub use joint::*;
pub use ray::*;
pub use traits::*;
pub use world::*;

#[cfg(test)]
mod tests {
	use super::*;
	use mochi_math::{Quat, Vec3};

	#[test]
	fn test_null_backend_body_lifecycle() {
		let config = PhysicsConfig::default();
		let mut world = PhysicsWorld::<NullPhysicsBackend>::new(config).expect("failed to init world");

		let body_def = BodyDef::dynamic()
			.with_position(Vec3::new(0.0, 10.0, 0.0))
			.with_mass(5.0);

		let body = world.create_body(&body_def).expect("failed to create body");
		assert!(world.is_body_valid(body));

		let (pos, rot) = world.body_transform(body).expect("failed to get transform");
		assert_eq!(pos, Vec3::new(0.0, 10.0, 0.0));
		assert_eq!(rot, Quat::IDENTITY);

		// Step simulation with gravity
		world.step_direct(1.0);
		let (new_pos, _) = world
			.body_transform(body)
			.expect("failed to get new transform");
		assert!(new_pos.y < 10.0, "Body should have fallen under gravity");

		// Clean up
		world.destroy_body(body).expect("failed to destroy body");
		assert!(!world.is_body_valid(body));
		assert!(world.body_transform(body).is_err());
	}

	#[test]
	fn test_collider_and_raycast_query() {
		let config = PhysicsConfig::default();
		let mut world = PhysicsWorld::<NullPhysicsBackend>::new(config).expect("failed to init world");

		let body_def = BodyDef::static_body().with_position(Vec3::new(0.0, 2.0, 5.0));
		let body = world.create_body(&body_def).expect("failed to create body");

		let collider_def = ColliderDef::new(ColliderShape::sphere(1.0));
		let collider = world
			.add_collider(body, &collider_def)
			.expect("failed to add collider");
		assert!(collider.is_valid());

		// Raycast directed straight at the body
		let ray = Ray::new(Vec3::new(0.0, 2.0, 0.0), Vec3::new(0.0, 0.0, 1.0), 10.0);
		let hit = world.raycast(&ray).expect("ray should hit collider");
		assert_eq!(hit.body, body);
		assert_eq!(hit.collider, collider);
		assert!((hit.distance - 4.0).abs() < 1e-4);

		// Raycast missing the body
		let miss_ray = Ray::new(Vec3::new(0.0, 20.0, 0.0), Vec3::new(0.0, 0.0, 1.0), 10.0);
		assert!(world.raycast(&miss_ray).is_none());
	}

	#[test]
	fn test_physics_accumulator_stepping() {
		let config = PhysicsConfig {
			fixed_timestep: 1.0 / 60.0,
			max_substeps: 4,
			..Default::default()
		};
		let mut world = PhysicsWorld::<NullPhysicsBackend>::new(config).expect("failed to init world");

		let body = world
			.create_body(&BodyDef::dynamic().with_position(Vec3::new(0.0, 5.0, 0.0)))
			.expect("failed to create body");

		// Frame with 0.033s delta time should trigger 2 fixed substeps (2 * 1/60s = 0.0333s)
		world.step_interpolated(0.034);
		let (pos, _) = world
			.body_transform(body)
			.expect("failed to read transform");
		assert!(pos.y < 5.0);
	}
}
