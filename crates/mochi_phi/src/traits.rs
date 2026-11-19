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
use crate::joint::{DistanceJointDef, JointHandle, RevoluteJointDef};
use crate::ray::{Ray, RaycastHit};
use crate::world::PhysicsConfig;


pub trait PhiBackend: 'static + Sized + Send + Sync {
	type World: PhiWorld;

	fn create_world(config: &PhysicsConfig) -> PhiResult<Self::World>;
}


pub trait PhiWorld: Send + Sync + 'static {
	fn step(&mut self, dt: f32);

	fn gravity(&self) -> Vec3<f32>;
	fn set_gravity(&mut self, gravity: Vec3<f32>);

	fn create_body(&mut self, def: &BodyDef) -> PhiResult<BodyHandle>;
	fn destroy_body(&mut self, handle: BodyHandle) -> PhiResult<()>;

	fn is_body_valid(&self, handle: BodyHandle) -> bool;

	fn body_transform(&self, handle: BodyHandle) -> PhiResult<(Vec3<f32>, Quat<f32>)>;

	fn set_body_transform(
		&mut self,
		handle: BodyHandle,
		pos: Vec3<f32>,
		rot: Quat<f32>,
	) -> PhiResult<()>;

	fn body_velocity(&self, handle: BodyHandle) -> PhiResult<(Vec3<f32>, Vec3<f32>)>;

	fn set_body_velocity(
		&mut self,
		handle: BodyHandle,
		linear: Vec3<f32>,
		angular: Vec3<f32>,
	) -> PhiResult<()>;

	fn apply_force(&mut self, handle: BodyHandle, force: Vec3<f32>) -> PhiResult<()>;

	fn apply_impulse(&mut self, handle: BodyHandle, impulse: Vec3<f32>) -> PhiResult<()>;

	fn apply_torque(&mut self, handle: BodyHandle, torque: Vec3<f32>) -> PhiResult<()>;

	fn apply_force_at_point(
		&mut self,
		handle: BodyHandle,
		force: Vec3<f32>,
		point: Vec3<f32>,
	) -> PhiResult<()>;

	fn apply_impulse_at_point(
		&mut self,
		handle: BodyHandle,
		impulse: Vec3<f32>,
		point: Vec3<f32>,
	) -> PhiResult<()>;

	fn add_collider(&mut self, body: BodyHandle, def: &ColliderDef) -> PhiResult<ColliderHandle>;

	fn remove_collider(&mut self, collider: ColliderHandle) -> PhiResult<()>;

	fn add_distance_joint(&mut self, def: &DistanceJointDef) -> PhiResult<JointHandle>;

	fn add_revolute_joint(&mut self, def: &RevoluteJointDef) -> PhiResult<JointHandle>;

	fn remove_joint(&mut self, joint: JointHandle) -> PhiResult<()>;

	fn raycast(&self, ray: &Ray) -> Option<RaycastHit>;

	fn raycast_all(&self, ray: &Ray) -> Vec<RaycastHit>;

	fn contact_manifolds(&self) -> Vec<ContactManifold>;

	fn contacts_for_body(&self, body: BodyHandle) -> Vec<ContactManifold>;
}
