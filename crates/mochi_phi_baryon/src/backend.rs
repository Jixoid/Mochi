use std::collections::HashMap;
use mochi_math::{Quat, Vec3};
use mochi_phi::{
	BodyDef, BodyHandle, BodyType, ColliderDef, ColliderHandle, ColliderShape, ContactManifold,
	DistanceJointDef, JointHandle, PhiBackend, PhiError, PhiResult, PhiWorld, PhysicsConfig, Ray,
	RaycastHit, RevoluteJointDef,
};
use crate::ffi::*;


/// Baryon C++ Physics Engine Backend for Mochi PHI
pub struct BaryonBackend;

impl PhiBackend for BaryonBackend {
	type World = BaryonWorld;

	fn create_world(config: &PhysicsConfig) -> PhiResult<Self::World> {
		let ptr = unsafe {
			baryon_sim_create(
				config.gravity.x,
				config.gravity.y,
				config.gravity.z,
				config.fixed_timestep,
			)
		};

		if ptr.is_null() {
			return Err(PhiError::BackendInitFailed(
				"Failed to instantiate Baryon Simulator".into(),
			));
		}

		Ok(BaryonWorld {
			ptr,
			colliders: HashMap::new(),
			next_collider_id: 1,
			joints: HashMap::new(),
			next_joint_id: 1,
		})
	}
}

pub struct BaryonWorld {
	ptr: BaryonSimulatorPtr,
	colliders: HashMap<ColliderHandle, (BodyHandle, ColliderDef)>,
	next_collider_id: u32,
	joints: HashMap<JointHandle, u32>,
	next_joint_id: u32,
}

unsafe impl Send for BaryonWorld {}
unsafe impl Sync for BaryonWorld {}

impl Drop for BaryonWorld {
	fn drop(&mut self) {
		if !self.ptr.is_null() {
			unsafe {
				baryon_sim_destroy(self.ptr);
			}
			self.ptr = std::ptr::null_mut();
		}
	}
}

impl PhiWorld for BaryonWorld {
	fn step(&mut self, dt: f32) {
		unsafe {
			baryon_sim_step(self.ptr, dt);
		}
	}

	fn gravity(&self) -> Vec3<f32> {
		let mut gx = 0.0f32;
		let mut gy = 0.0f32;
		let mut gz = 0.0f32;
		unsafe {
			baryon_sim_get_gravity(self.ptr, &mut gx, &mut gy, &mut gz);
		}
		Vec3::new(gx, gy, gz)
	}

	fn set_gravity(&mut self, gravity: Vec3<f32>) {
		unsafe {
			baryon_sim_set_gravity(self.ptr, gravity.x, gravity.y, gravity.z);
		}
	}

	fn create_body(&mut self, def: &BodyDef) -> PhiResult<BodyHandle> {
		let b_type = match def.body_type {
			BodyType::Static => 0,
			BodyType::Kinematic => 1,
			BodyType::Dynamic => 2,
		};

		let id = unsafe {
			baryon_sim_create_body_box(
				self.ptr,
				def.position.x,
				def.position.y,
				def.position.z,
				def.rotation.x,
				def.rotation.y,
				def.rotation.z,
				def.rotation.w,
				0.5,
				0.5,
				0.5,
				b_type,
				def.mass,
				0.4,
				0.0,
				def.use_ccd,
			)
		};

		if id == 0xFFFFFFFF {
			return Err(PhiError::SimulationError(
				"Baryon failed to create body".into(),
			));
		}

		if def.linear_velocity != Vec3::ZERO || def.angular_velocity != Vec3::ZERO {
			unsafe {
				baryon_sim_set_velocity(
					self.ptr,
					id,
					def.linear_velocity.x,
					def.linear_velocity.y,
					def.linear_velocity.z,
					def.angular_velocity.x,
					def.angular_velocity.y,
					def.angular_velocity.z,
				);
			}
		}

		Ok(BodyHandle(id))
	}

	fn destroy_body(&mut self, handle: BodyHandle) -> PhiResult<()> {
		let success = unsafe { baryon_sim_destroy_body(self.ptr, handle.0) };
		if success {
			self.colliders.retain(|_, (b, _)| *b != handle);
			Ok(())
		} else {
			Err(PhiError::BodyNotFound(handle))
		}
	}

	fn is_body_valid(&self, handle: BodyHandle) -> bool {
		unsafe { baryon_sim_is_body_alive(self.ptr, handle.0) }
	}

	fn body_transform(&self, handle: BodyHandle) -> PhiResult<(Vec3<f32>, Quat<f32>)> {
		let mut pos = [0.0f32; 3];
		let mut rot = [0.0f32; 4];
		let success =
			unsafe { baryon_sim_get_transform(self.ptr, handle.0, pos.as_mut_ptr(), rot.as_mut_ptr()) };

		if success {
			Ok((
				Vec3::new(pos[0], pos[1], pos[2]),
				Quat::new(rot[0], rot[1], rot[2], rot[3]),
			))
		} else {
			Err(PhiError::BodyNotFound(handle))
		}
	}

	fn set_body_transform(
		&mut self,
		handle: BodyHandle,
		pos: Vec3<f32>,
		rot: Quat<f32>,
	) -> PhiResult<()> {
		let success = unsafe {
			baryon_sim_set_transform(
				self.ptr, handle.0, pos.x, pos.y, pos.z, rot.x, rot.y, rot.z, rot.w,
			)
		};
		if success {
			Ok(())
		} else {
			Err(PhiError::BodyNotFound(handle))
		}
	}

	fn body_velocity(&self, handle: BodyHandle) -> PhiResult<(Vec3<f32>, Vec3<f32>)> {
		let mut lin = [0.0f32; 3];
		let mut ang = [0.0f32; 3];
		let success =
			unsafe { baryon_sim_get_velocity(self.ptr, handle.0, lin.as_mut_ptr(), ang.as_mut_ptr()) };
		if success {
			Ok((
				Vec3::new(lin[0], lin[1], lin[2]),
				Vec3::new(ang[0], ang[1], ang[2]),
			))
		} else {
			Err(PhiError::BodyNotFound(handle))
		}
	}

	fn set_body_velocity(
		&mut self,
		handle: BodyHandle,
		linear: Vec3<f32>,
		angular: Vec3<f32>,
	) -> PhiResult<()> {
		let success = unsafe {
			baryon_sim_set_velocity(
				self.ptr, handle.0, linear.x, linear.y, linear.z, angular.x, angular.y, angular.z,
			)
		};
		if success {
			Ok(())
		} else {
			Err(PhiError::BodyNotFound(handle))
		}
	}

	fn apply_force(&mut self, handle: BodyHandle, force: Vec3<f32>) -> PhiResult<()> {
		let success = unsafe { baryon_sim_apply_force(self.ptr, handle.0, force.x, force.y, force.z) };
		if success {
			Ok(())
		} else {
			Err(PhiError::BodyNotFound(handle))
		}
	}

	fn apply_impulse(&mut self, handle: BodyHandle, impulse: Vec3<f32>) -> PhiResult<()> {
		let success =
			unsafe { baryon_sim_apply_impulse(self.ptr, handle.0, impulse.x, impulse.y, impulse.z) };
		if success {
			Ok(())
		} else {
			Err(PhiError::BodyNotFound(handle))
		}
	}

	fn apply_torque(&mut self, handle: BodyHandle, torque: Vec3<f32>) -> PhiResult<()> {
		let success =
			unsafe { baryon_sim_apply_torque(self.ptr, handle.0, torque.x, torque.y, torque.z) };
		if success {
			Ok(())
		} else {
			Err(PhiError::BodyNotFound(handle))
		}
	}

	fn apply_force_at_point(
		&mut self,
		handle: BodyHandle,
		force: Vec3<f32>,
		_point: Vec3<f32>,
	) -> PhiResult<()> {
		self.apply_force(handle, force)
	}

	fn apply_impulse_at_point(
		&mut self,
		handle: BodyHandle,
		impulse: Vec3<f32>,
		_point: Vec3<f32>,
	) -> PhiResult<()> {
		self.apply_impulse(handle, impulse)
	}

	fn add_collider(&mut self, body: BodyHandle, def: &ColliderDef) -> PhiResult<ColliderHandle> {
		if !self.is_body_valid(body) {
			return Err(PhiError::BodyNotFound(body));
		}

		match &def.shape {
			ColliderShape::Box { half_extents } => unsafe {
				baryon_sim_set_collider_box(
					self.ptr,
					body.0,
					half_extents.x,
					half_extents.y,
					half_extents.z,
					def.material.friction,
					def.material.restitution,
				);
			},
			ColliderShape::Sphere { radius } => unsafe {
				baryon_sim_set_collider_sphere(
					self.ptr,
					body.0,
					*radius,
					def.material.friction,
					def.material.restitution,
				);
			},
			ColliderShape::Capsule { radius, height } => unsafe {
				baryon_sim_set_collider_capsule(
					self.ptr,
					body.0,
					*radius,
					*height,
					def.material.friction,
					def.material.restitution,
				);
			},
			_ => {}
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

	fn add_distance_joint(&mut self, def: &DistanceJointDef) -> PhiResult<JointHandle> {
		let c_id = unsafe {
			baryon_sim_create_distance_constraint(
				self.ptr,
				def.body_a.0,
				def.body_b.0,
				def.target_distance,
			)
		};
		if c_id == 0xFFFFFFFF {
			return Err(PhiError::SimulationError(
				"Failed to create distance constraint in Baryon".into(),
			));
		}
		let handle = JointHandle(self.next_joint_id);
		self.next_joint_id += 1;
		self.joints.insert(handle, c_id);
		Ok(handle)
	}

	fn add_revolute_joint(&mut self, def: &RevoluteJointDef) -> PhiResult<JointHandle> {
		let c_id = unsafe {
			baryon_sim_create_revolute_constraint(
				self.ptr,
				def.body_a.0,
				def.body_b.0,
				def.local_anchor_a.x,
				def.local_anchor_a.y,
				def.local_anchor_a.z,
				def.local_anchor_b.x,
				def.local_anchor_b.y,
				def.local_anchor_b.z,
				def.local_axis_a.x,
				def.local_axis_a.y,
				def.local_axis_a.z,
				def.local_axis_b.x,
				def.local_axis_b.y,
				def.local_axis_b.z,
			)
		};
		if c_id == 0xFFFFFFFF {
			return Err(PhiError::SimulationError(
				"Failed to create revolute constraint in Baryon".into(),
			));
		}
		let handle = JointHandle(self.next_joint_id);
		self.next_joint_id += 1;
		self.joints.insert(handle, c_id);
		Ok(handle)
	}

	fn remove_joint(&mut self, joint: JointHandle) -> PhiResult<()> {
		if self.joints.remove(&joint).is_some() {
			Ok(())
		} else {
			Err(PhiError::JointNotFound(joint))
		}
	}

	fn raycast(&self, ray: &Ray) -> Option<RaycastHit> {
		let mut hit_body = 0u32;
		let mut hit_pos = [0.0f32; 3];
		let mut hit_norm = [0.0f32; 3];
		let mut hit_dist = 0.0f32;

		let has_hit = unsafe {
			baryon_sim_raycast(
				self.ptr,
				ray.origin.x,
				ray.origin.y,
				ray.origin.z,
				ray.direction.x,
				ray.direction.y,
				ray.direction.z,
				ray.max_distance,
				&mut hit_body,
				hit_pos.as_mut_ptr(),
				hit_norm.as_mut_ptr(),
				&mut hit_dist,
			)
		};

		if has_hit {
			Some(RaycastHit {
				body: BodyHandle(hit_body),
				collider: ColliderHandle::default(),
				point: Vec3::new(hit_pos[0], hit_pos[1], hit_pos[2]),
				normal: Vec3::new(hit_norm[0], hit_norm[1], hit_norm[2]),
				distance: hit_dist,
			})
		} else {
			None
		}
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
