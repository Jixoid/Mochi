//! # Mochi PHI Baryon Driver
//!
//! C-ABI bridge and driver implementation integrating the C++23 Baryon Physics Engine into Mochi PHI

pub mod backend;
pub mod ffi;

pub use backend::*;

#[cfg(test)]
mod tests {
	use super::*;
	use mochi_math::Vec3;
	use mochi_phi::{BodyDef, PhysicsConfig, PhysicsWorld, Ray};

	#[test]
	fn test_baryon_world_lifecycle_and_gravity() {
		let config = PhysicsConfig {
			gravity: Vec3::new(0.0, -9.81, 0.0),
			fixed_timestep: 1.0 / 60.0,
			..Default::default()
		};

		let mut world =
			PhysicsWorld::<BaryonBackend>::new(config).expect("failed to create Baryon world");

		let body_def = BodyDef::dynamic()
			.with_position(Vec3::new(0.0, 10.0, 0.0))
			.with_mass(2.0);

		let body = world.create_body(&body_def).expect("failed to create body");
		assert!(world.is_body_valid(body));

		let (init_pos, _) = world
			.body_transform(body)
			.expect("failed to read initial pos");
		assert_eq!(init_pos, Vec3::new(0.0, 10.0, 0.0));

		// Step physics multiple times
		for _ in 0..10 {
			world.step_direct(1.0 / 60.0);
		}

		let (after_pos, _) = world
			.body_transform(body)
			.expect("failed to read pos after step");
		assert!(
			after_pos.y < 10.0,
			"Body should have fallen under Baryon gravity: y = {}",
			after_pos.y
		);

		world.destroy_body(body).expect("failed to destroy body");
		assert!(!world.is_body_valid(body));
	}

	#[test]
	fn test_baryon_velocity_and_force() {
		let config = PhysicsConfig::default();
		let mut world =
			PhysicsWorld::<BaryonBackend>::new(config).expect("failed to create Baryon world");

		let body_def = BodyDef::dynamic().with_position(Vec3::new(0.0, 0.0, 0.0));
		let body = world.create_body(&body_def).expect("failed to create body");

		world
			.apply_impulse(body, Vec3::new(10.0, 0.0, 0.0))
			.expect("failed to apply impulse");

		let (lin_vel, _) = world.body_velocity(body).expect("failed to get velocity");
		assert!(
			lin_vel.x > 0.0,
			"Velocity should be positive in X: {}",
			lin_vel.x
		);

		world.step_direct(1.0 / 60.0);
		let (pos, _) = world.body_transform(body).expect("failed to get transform");
		assert!(pos.x > 0.0, "Body should have moved along X: {}", pos.x);

		world.destroy_body(body).expect("failed to destroy body");
	}

	#[test]
	fn test_baryon_raycast() {
		let config = PhysicsConfig::default();
		let mut world =
			PhysicsWorld::<BaryonBackend>::new(config).expect("failed to create Baryon world");

		// Spawn a static box at (0, 0, 5)
		let body_def = BodyDef::static_body().with_position(Vec3::new(0.0, 0.0, 5.0));
		let body = world
			.create_body(&body_def)
			.expect("failed to create static box");

		let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), 20.0);
		let hit = world.raycast(&ray);
		assert!(hit.is_some(), "Baryon raycast should hit static box");

		let hit = hit.unwrap();
		assert_eq!(hit.body, body);
		assert!(hit.distance > 0.0);

		world.destroy_body(body).expect("failed to destroy body");
	}

	#[test]
	fn test_baryon_cubes_collision_and_stack() {
		let config = PhysicsConfig {
			gravity: Vec3::new(0.0, -9.81, 0.0),
			fixed_timestep: 1.0 / 60.0,
			..Default::default()
		};

		let mut world =
			PhysicsWorld::<BaryonBackend>::new(config).expect("failed to create Baryon world");

		// Static ground cube: pos = (0, 0, 0), half_extents = (10, 0.5, 10)
		let ground_body = world
			.create_body(&BodyDef::static_body().with_position(Vec3::new(0.0, 0.0, 0.0)))
			.expect("failed to create ground");
		world
			.add_collider(
				ground_body,
				&mochi_phi::ColliderDef::new(mochi_phi::ColliderShape::box_shape(10.0, 0.5, 10.0)),
			)
			.expect("failed to add ground collider");

		// Dynamic falling cubes at y = 2.5, 5.0, 7.5
		let mut cubes = Vec::new();
		for i in 1..=3 {
			let y = i as f32 * 2.5;
			let body = world
				.create_body(
					&BodyDef::dynamic()
						.with_position(Vec3::new(0.0, y, 0.0))
						.with_mass(1.0),
				)
				.expect("failed to create cube");
			world
				.add_collider(
					body,
					&mochi_phi::ColliderDef::new(mochi_phi::ColliderShape::box_shape(0.5, 0.5, 0.5)),
				)
				.expect("failed to add cube collider");
			cubes.push(body);
		}

		// Step physics for 2 seconds (120 steps)
		for _ in 0..120 {
			world.step_direct(1.0 / 60.0);
		}

		// Ground should stay at 0.0
		let (ground_pos, _) = world.body_transform(ground_body).unwrap();
		assert_eq!(ground_pos, Vec3::new(0.0, 0.0, 0.0));

		// All cubes should have fallen and be supported above ground
		for (i, &cube) in cubes.iter().enumerate() {
			let (pos, _) = world.body_transform(cube).unwrap();
			assert!(
				pos.y >= 0.5,
				"Cube {} at y = {} should not penetrate through ground (y >= 0.5)",
				i,
				pos.y
			);
			assert!(
				pos.y < (i as f32 + 1.0) * 2.5,
				"Cube {} at y = {} should have fallen downward",
				i,
				pos.y
			);
		}
	}
}
