/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;
use mochi_math::{Quat, Vec3};
use crate::body::{BodyDef, BodyHandle};
use crate::collider::{ColliderDef, ColliderHandle, ContactManifold};
use crate::error::PhiResult;
use crate::joint::{DistanceJointDef, JointHandle, RevoluteJointDef, SpringJointDef};
use crate::ray::{Ray, RaycastHit};
use crate::traits::{PhiBackend, PhiWorld};


#[derive(Clone, Debug, PartialEq)]
pub struct PhysicsConfig {
	pub gravity: Vec3<f32>,
	pub fixed_timestep: f32,
	pub max_substeps: u32,
	pub velocity_iterations: u32,
	pub position_iterations: u32,
	pub allow_sleep: bool,
}

impl Default for PhysicsConfig {
	fn default() -> Self {
		Self {
			gravity: Vec3::new(0.0, -9.81, 0.0),
			fixed_timestep: 1.0 / 60.0,
			max_substeps: 4,
			velocity_iterations: 8,
			position_iterations: 3,
			allow_sleep: true,
		}
	}
}


pub struct PhysicsWorld<B: PhiBackend> {
	inner: B::World,
	config: PhysicsConfig,
	accumulator: f32,
}

impl<B: PhiBackend> PhysicsWorld<B> {
	pub fn new(config: PhysicsConfig) -> PhiResult<Self> {
		let inner = B::create_world(&config)?;
		Ok(Self {
			inner,
			config,
			accumulator: 0.0,
		})
	}

	pub fn config(&self) -> &PhysicsConfig {
		&self.config
	}

	pub fn inner(&self) -> &B::World {
		&self.inner
	}

	pub fn inner_mut(&mut self) -> &mut B::World {
		&mut self.inner
	}

	pub fn step_direct(&mut self, dt: f32) {
		self.inner.step(dt);
	}

	#[inline]
	pub fn step(&mut self, dt: f32) {
		self.step_interpolated(dt);
	}

	pub fn step_interpolated(&mut self, delta_time: f32) {
		let clamped_dt = delta_time.min(0.1);
		self.accumulator += clamped_dt;

		let dt = self.config.fixed_timestep;
		let mut steps = 0;

		while self.accumulator >= dt && steps < self.config.max_substeps {
			self.inner.step(dt);
			self.accumulator -= dt;
			steps += 1;
		}

		if steps >= self.config.max_substeps {
			self.accumulator = 0.0;
		}
	}

	pub fn clear_contacts(&mut self) {
		self.accumulator = 0.0;
		self.inner.clear_contacts();
	}

	pub fn gravity(&self) -> Vec3<f32> {
		self.inner.gravity()
	}

	pub fn set_gravity(&mut self, gravity: Vec3<f32>) {
		self.inner.set_gravity(gravity);
	}

	pub fn create_body(&mut self, def: &BodyDef) -> PhiResult<BodyHandle> {
		self.inner.create_body(def)
	}

	pub fn destroy_body(&mut self, handle: BodyHandle) -> PhiResult<()> {
		self.inner.destroy_body(handle)
	}

	pub fn is_body_valid(&self, handle: BodyHandle) -> bool {
		self.inner.is_body_valid(handle)
	}

	pub fn body_transform(&self, handle: BodyHandle) -> PhiResult<(Vec3<f32>, Quat<f32>)> {
		self.inner.body_transform(handle)
	}

	pub fn set_body_transform(
		&mut self,
		handle: BodyHandle,
		pos: Vec3<f32>,
		rot: Quat<f32>,
	) -> PhiResult<()> {
		self.inner.set_body_transform(handle, pos, rot)
	}

	pub fn body_velocity(&self, handle: BodyHandle) -> PhiResult<(Vec3<f32>, Vec3<f32>)> {
		self.inner.body_velocity(handle)
	}

	pub fn set_body_velocity(
		&mut self,
		handle: BodyHandle,
		linear: Vec3<f32>,
		angular: Vec3<f32>,
	) -> PhiResult<()> {
		self.inner.set_body_velocity(handle, linear, angular)
	}

	pub fn apply_force(&mut self, handle: BodyHandle, force: Vec3<f32>) -> PhiResult<()> {
		self.inner.apply_force(handle, force)
	}

	pub fn apply_impulse(&mut self, handle: BodyHandle, impulse: Vec3<f32>) -> PhiResult<()> {
		self.inner.apply_impulse(handle, impulse)
	}

	pub fn apply_torque(&mut self, handle: BodyHandle, torque: Vec3<f32>) -> PhiResult<()> {
		self.inner.apply_torque(handle, torque)
	}

	pub fn add_collider(&mut self, body: BodyHandle, def: &ColliderDef) -> PhiResult<ColliderHandle> {
		self.inner.add_collider(body, def)
	}

	pub fn remove_collider(&mut self, collider: ColliderHandle) -> PhiResult<()> {
		self.inner.remove_collider(collider)
	}

	pub fn add_distance_joint(&mut self, def: &DistanceJointDef) -> PhiResult<JointHandle> {
		self.inner.add_distance_joint(def)
	}

	pub fn add_revolute_joint(&mut self, def: &RevoluteJointDef) -> PhiResult<JointHandle> {
		self.inner.add_revolute_joint(def)
	}

	pub fn add_spring_joint(&mut self, def: &SpringJointDef) -> PhiResult<JointHandle> {
		self.inner.add_spring_joint(def)
	}

	pub fn remove_joint(&mut self, joint: JointHandle) -> PhiResult<()> {
		self.inner.remove_joint(joint)
	}

	pub fn raycast(&self, ray: &Ray) -> Option<RaycastHit> {
		self.inner.raycast(ray)
	}

	pub fn raycast_all(&self, ray: &Ray) -> Vec<RaycastHit> {
		self.inner.raycast_all(ray)
	}

	pub fn contact_manifolds(&self) -> Vec<ContactManifold> {
		self.inner.contact_manifolds()
	}

	pub fn contacts_for_body(&self, body: BodyHandle) -> Vec<ContactManifold> {
		self.inner.contacts_for_body(body)
	}
}


pub struct NullPhysicsBackend;
pub type DummyPhysicsBackend = NullPhysicsBackend;

#[derive(Clone, Debug)]
pub struct MockBody {
	pub position: Vec3<f32>,
	pub rotation: Quat<f32>,
	pub linear_velocity: Vec3<f32>,
	pub angular_velocity: Vec3<f32>,
}

pub struct NullPhysicsWorld {
	gravity: Vec3<f32>,
	next_body_id: u32,
	next_collider_id: u32,
	next_joint_id: u32,
	bodies: HashMap<BodyHandle, MockBody>,
	colliders: HashMap<ColliderHandle, (BodyHandle, ColliderDef)>,
}

pub type DummyPhysicsWorld = NullPhysicsWorld;

impl PhiBackend for NullPhysicsBackend {
	type World = NullPhysicsWorld;

	fn create_world(config: &PhysicsConfig) -> PhiResult<Self::World> {
		Ok(NullPhysicsWorld {
			gravity: config.gravity,
			next_body_id: 1,
			next_collider_id: 1,
			next_joint_id: 1,
			bodies: HashMap::new(),
			colliders: HashMap::new(),
		})
	}
}

impl PhiWorld for NullPhysicsWorld {
	fn step(&mut self, _dt: f32) {
		// Completely empty dummy physics step
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
			MockBody {
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
			Err(crate::error::PhiError::BodyNotFound(handle))
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
			.ok_or(crate::error::PhiError::BodyNotFound(handle))
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
			Err(crate::error::PhiError::BodyNotFound(handle))
		}
	}

	fn body_velocity(&self, handle: BodyHandle) -> PhiResult<(Vec3<f32>, Vec3<f32>)> {
		self
			.bodies
			.get(&handle)
			.map(|b| (b.linear_velocity, b.angular_velocity))
			.ok_or(crate::error::PhiError::BodyNotFound(handle))
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
			Err(crate::error::PhiError::BodyNotFound(handle))
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
			return Err(crate::error::PhiError::BodyNotFound(body));
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
			Err(crate::error::PhiError::ColliderNotFound(collider))
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
					crate::collider::ColliderShape::Sphere { radius } => *radius,
					crate::collider::ColliderShape::Box { half_extents } => half_extents.x.max(half_extents.y).max(half_extents.z),
					crate::collider::ColliderShape::Capsule { radius, height } => radius + height * 0.5,
					_ => 1.0,
				};
				let oc = ray.origin - center;
				let b = oc.dot(&ray.direction);
				let c = oc.dot(&oc) - radius * radius;
				let discriminant = b * b - c;

				if discriminant > 0.0 {
					let t = -b - discriminant.sqrt();
					if t > 0.0 && t < ray.max_distance && closest_hit.as_ref().is_none_or(|h| t < h.distance)
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
