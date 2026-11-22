/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


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
