/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;
use mochi_math::{Quat, Vec3};
use mochi_phi::{
	BodyDef, BodyHandle, ColliderDef, ColliderHandle, ContactManifold, DistanceJointDef,
	JointHandle, PhiBackend, PhiError, PhiResult, PhiWorld, PhysicsConfig, Ray, RaycastHit,
	RevoluteJointDef, SpringJointDef,
};


pub struct TachyonBackend;

#[derive(Clone, Debug)]
pub struct TachyonBody {
	pub position: Vec3<f32>,
	pub rotation: Quat<f32>,
	pub linear_velocity: Vec3<f32>,
	pub angular_velocity: Vec3<f32>,
}

pub struct TachyonWorld {
	gravity: Vec3<f32>,
	next_body_id: u32,
	next_collider_id: u32,
	next_joint_id: u32,
	bodies: HashMap<BodyHandle, TachyonBody>,
	colliders: HashMap<ColliderHandle, (BodyHandle, ColliderDef)>,
}


impl PhiBackend for TachyonBackend {
	type World = TachyonWorld;

	fn create_world(config: &PhysicsConfig) -> PhiResult<Self::World> {
		Ok(TachyonWorld {
			gravity: config.gravity,
			next_body_id: 1,
			next_collider_id: 1,
			next_joint_id: 1,
			bodies: HashMap::new(),
			colliders: HashMap::new(),
		})
	}
}

impl PhiWorld for TachyonWorld {
	fn step(&mut self, _dt: f32) {
		// Tachyon simulation step
	}

	fn gravity(&self) -> Vec3<f32> {
		self.gravity
	}

	fn set_gravity(&mut self, gravity: Vec3<f32>) {
		self.gravity = gravity;
	}

	fn create_body(&mut self, def: &BodyDef) -> PhiResult<BodyHandle> {
		let handle = BodyHandle(self.next_body_id);
		self.next_body_id += 1;
		self.bodies.insert(
			handle,
			TachyonBody {
				position: def.position,
				rotation: def.rotation,
				linear_velocity: def.linear_velocity,
				angular_velocity: def.angular_velocity,
			},
		);
		Ok(handle)
	}

	fn destroy_body(&mut self, handle: BodyHandle) -> PhiResult<()> {
		if self.bodies.remove(&handle).is_some() {
			self.colliders.retain(|_, (b, _)| *b != handle);
			Ok(())
		} else {
			Err(PhiError::BodyNotFound(handle))
		}
	}

	fn is_body_valid(&self, handle: BodyHandle) -> bool {
		self.bodies.contains_key(&handle)
	}

	fn body_transform(&self, handle: BodyHandle) -> PhiResult<(Vec3<f32>, Quat<f32>)> {
		self
			.bodies
			.get(&handle)
			.map(|b| (b.position, b.rotation))
			.ok_or(PhiError::BodyNotFound(handle))
	}

	fn set_body_transform(
		&mut self,
		handle: BodyHandle,
		pos: Vec3<f32>,
		rot: Quat<f32>,
	) -> PhiResult<()> {
		if let Some(entry) = self.bodies.get_mut(&handle) {
			entry.position = pos;
			entry.rotation = rot;
			Ok(())
		} else {
			Err(PhiError::BodyNotFound(handle))
		}
	}

	fn body_velocity(&self, handle: BodyHandle) -> PhiResult<(Vec3<f32>, Vec3<f32>)> {
		self
			.bodies
			.get(&handle)
			.map(|b| (b.linear_velocity, b.angular_velocity))
			.ok_or(PhiError::BodyNotFound(handle))
	}

	fn set_body_velocity(
		&mut self,
		handle: BodyHandle,
		linear: Vec3<f32>,
		angular: Vec3<f32>,
	) -> PhiResult<()> {
		if let Some(entry) = self.bodies.get_mut(&handle) {
			entry.linear_velocity = linear;
			entry.angular_velocity = angular;
			Ok(())
		} else {
			Err(PhiError::BodyNotFound(handle))
		}
	}

	fn apply_force(&mut self, _handle: BodyHandle, _force: Vec3<f32>) -> PhiResult<()> {
		Ok(())
	}

	fn apply_impulse(&mut self, _handle: BodyHandle, _impulse: Vec3<f32>) -> PhiResult<()> {
		Ok(())
	}

	fn apply_torque(&mut self, _handle: BodyHandle, _torque: Vec3<f32>) -> PhiResult<()> {
		Ok(())
	}

	fn apply_force_at_point(
		&mut self,
		_handle: BodyHandle,
		_force: Vec3<f32>,
		_point: Vec3<f32>,
	) -> PhiResult<()> {
		Ok(())
	}

	fn apply_impulse_at_point(
		&mut self,
		_handle: BodyHandle,
		_impulse: Vec3<f32>,
		_point: Vec3<f32>,
	) -> PhiResult<()> {
		Ok(())
	}

	fn add_collider(&mut self, body: BodyHandle, def: &ColliderDef) -> PhiResult<ColliderHandle> {
		if !self.is_body_valid(body) {
			return Err(PhiError::BodyNotFound(body));
		}
		let handle = ColliderHandle(self.next_collider_id);
		self.next_collider_id += 1;
		self.colliders.insert(handle, (body, def.clone()));
		Ok(handle)
	}

	fn remove_collider(&mut self, collider: ColliderHandle) -> PhiResult<()> {
		if self.colliders.remove(&collider).is_some() {
			Ok(())
		} else {
			Err(PhiError::ColliderNotFound(collider))
		}
	}

	fn add_distance_joint(&mut self, _def: &DistanceJointDef) -> PhiResult<JointHandle> {
		let handle = JointHandle(self.next_joint_id);
		self.next_joint_id += 1;
		Ok(handle)
	}

	fn add_revolute_joint(&mut self, _def: &RevoluteJointDef) -> PhiResult<JointHandle> {
		let handle = JointHandle(self.next_joint_id);
		self.next_joint_id += 1;
		Ok(handle)
	}

	fn add_spring_joint(&mut self, _def: &SpringJointDef) -> PhiResult<JointHandle> {
		let handle = JointHandle(self.next_joint_id);
		self.next_joint_id += 1;
		Ok(handle)
	}

	fn remove_joint(&mut self, _joint: JointHandle) -> PhiResult<()> {
		Ok(())
	}

	fn raycast(&self, ray: &Ray) -> Option<RaycastHit> {
		let mut closest_hit: Option<RaycastHit> = None;

		for (&collider_h, (body_h, collider_def)) in &self.colliders {
			if let Some(body) = self.bodies.get(body_h) {
				let center = body.position + collider_def.offset_position;
				let radius = match &collider_def.shape {
					mochi_phi::ColliderShape::Sphere { radius } => *radius,
					mochi_phi::ColliderShape::Box { half_extents } => {
						half_extents.x.max(half_extents.y).max(half_extents.z)
					}
					mochi_phi::ColliderShape::Capsule { radius, height } => radius + height * 0.5,
					_ => 1.0,
				};
				let oc = ray.origin - center;
				let b = oc.dot(&ray.direction);
				let c = oc.dot(&oc) - radius * radius;
				let discriminant = b * b - c;

				if discriminant > 0.0 {
					let t = -b - discriminant.sqrt();
					if t > 0.0
						&& t < ray.max_distance
						&& closest_hit.as_ref().is_none_or(|h| t < h.distance)
					{
						closest_hit = Some(RaycastHit {
							body: *body_h,
							collider: collider_h,
							point: ray.point_at(t),
							normal: (ray.point_at(t) - center).normalize(),
							distance: t,
						});
					}
				}
			}
		}

		closest_hit
	}

	fn raycast_all(&self, ray: &Ray) -> Vec<RaycastHit> {
		self.raycast(ray).into_iter().collect()
	}

	fn contact_manifolds(&self) -> Vec<ContactManifold> {
		Vec::new()
	}

	fn contacts_for_body(&self, _body: BodyHandle) -> Vec<ContactManifold> {
		Vec::new()
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use mochi_phi::{ColliderShape, PhysicsWorld};

	#[test]
	fn test_tachyon_backend_body_lifecycle() {
		let config = PhysicsConfig::default();
		let mut world =
			PhysicsWorld::<TachyonBackend>::new(config).expect("failed to init Tachyon world");

		let body_def = BodyDef::dynamic().with_position(Vec3::new(1.0, 2.0, 3.0));
		let body = world.create_body(&body_def).expect("failed to create body");
		assert!(world.is_body_valid(body));

		let (pos, rot) = world.body_transform(body).expect("failed to get transform");
		assert_eq!(pos, Vec3::new(1.0, 2.0, 3.0));
		assert_eq!(rot, Quat::IDENTITY);

		world.step_direct(1.0);
		let (new_pos, _) = world.body_transform(body).expect("get transform after step");
		assert_eq!(new_pos, Vec3::new(1.0, 2.0, 3.0));

		world.destroy_body(body).expect("destroy body");
		assert!(!world.is_body_valid(body));
	}

	#[test]
	fn test_tachyon_collider_and_raycast_query() {
		let config = PhysicsConfig::default();
		let mut world =
			PhysicsWorld::<TachyonBackend>::new(config).expect("failed to init Tachyon world");

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
	fn test_tachyon_physics_accumulator_stepping() {
		let config = PhysicsConfig {
			fixed_timestep: 1.0 / 60.0,
			max_substeps: 4,
			..Default::default()
		};
		let mut world =
			PhysicsWorld::<TachyonBackend>::new(config).expect("failed to init Tachyon world");

		let body = world
			.create_body(&BodyDef::dynamic().with_position(Vec3::new(0.0, 5.0, 0.0)))
			.expect("failed to create body");

		world.step_interpolated(0.034);
		let (pos, _) = world
			.body_transform(body)
			.expect("failed to read transform");
		assert_eq!(pos, Vec3::new(0.0, 5.0, 0.0));
	}
}
