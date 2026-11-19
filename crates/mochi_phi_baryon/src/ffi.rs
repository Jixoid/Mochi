use std::os::raw::c_int;


#[repr(C)]
pub struct BaryonSimulatorOpaque {
	_private: [u8; 0],
}

pub type BaryonSimulatorPtr = *mut BaryonSimulatorOpaque;

unsafe extern "C" {
	pub fn baryon_sim_create(
		gravity_x: f32,
		gravity_y: f32,
		gravity_z: f32,
		fixed_dt: f32,
	) -> BaryonSimulatorPtr;

	pub fn baryon_sim_destroy(sim: BaryonSimulatorPtr);
	pub fn baryon_sim_step(sim: BaryonSimulatorPtr, dt: f32);
	pub fn baryon_sim_set_gravity(sim: BaryonSimulatorPtr, gx: f32, gy: f32, gz: f32);
	pub fn baryon_sim_get_gravity(
		sim: BaryonSimulatorPtr,
		out_gx: *mut f32,
		out_gy: *mut f32,
		out_gz: *mut f32,
	);

	pub fn baryon_sim_create_body_box(
		sim: BaryonSimulatorPtr,
		px: f32,
		py: f32,
		pz: f32,
		qx: f32,
		qy: f32,
		qz: f32,
		qw: f32,
		hx: f32,
		hy: f32,
		hz: f32,
		body_type: c_int,
		mass: f32,
		friction: f32,
		restitution: f32,
		use_ccd: bool,
	) -> u32;

	pub fn baryon_sim_create_body_sphere(
		sim: BaryonSimulatorPtr,
		px: f32,
		py: f32,
		pz: f32,
		qx: f32,
		qy: f32,
		qz: f32,
		qw: f32,
		radius: f32,
		body_type: c_int,
		mass: f32,
		friction: f32,
		restitution: f32,
		use_ccd: bool,
	) -> u32;

	pub fn baryon_sim_create_body_capsule(
		sim: BaryonSimulatorPtr,
		px: f32,
		py: f32,
		pz: f32,
		qx: f32,
		qy: f32,
		qz: f32,
		qw: f32,
		radius: f32,
		height: f32,
		body_type: c_int,
		mass: f32,
		friction: f32,
		restitution: f32,
		use_ccd: bool,
	) -> u32;

	pub fn baryon_sim_destroy_body(sim: BaryonSimulatorPtr, body_id: u32) -> bool;
	pub fn baryon_sim_is_body_alive(sim: BaryonSimulatorPtr, body_id: u32) -> bool;

	pub fn baryon_sim_set_collider_box(
		sim: BaryonSimulatorPtr,
		body_id: u32,
		hx: f32,
		hy: f32,
		hz: f32,
		friction: f32,
		restitution: f32,
	) -> bool;

	pub fn baryon_sim_set_collider_sphere(
		sim: BaryonSimulatorPtr,
		body_id: u32,
		radius: f32,
		friction: f32,
		restitution: f32,
	) -> bool;

	pub fn baryon_sim_set_collider_capsule(
		sim: BaryonSimulatorPtr,
		body_id: u32,
		radius: f32,
		height: f32,
		friction: f32,
		restitution: f32,
	) -> bool;

	pub fn baryon_sim_get_transform(
		sim: BaryonSimulatorPtr,
		body_id: u32,
		out_pos: *mut f32,
		out_rot: *mut f32,
	) -> bool;

	pub fn baryon_sim_set_transform(
		sim: BaryonSimulatorPtr,
		body_id: u32,
		px: f32,
		py: f32,
		pz: f32,
		qx: f32,
		qy: f32,
		qz: f32,
		qw: f32,
	) -> bool;

	pub fn baryon_sim_get_velocity(
		sim: BaryonSimulatorPtr,
		body_id: u32,
		out_lin: *mut f32,
		out_ang: *mut f32,
	) -> bool;

	pub fn baryon_sim_set_velocity(
		sim: BaryonSimulatorPtr,
		body_id: u32,
		lx: f32,
		ly: f32,
		lz: f32,
		ax: f32,
		ay: f32,
		az: f32,
	) -> bool;

	pub fn baryon_sim_apply_force(
		sim: BaryonSimulatorPtr,
		body_id: u32,
		fx: f32,
		fy: f32,
		fz: f32,
	) -> bool;
	pub fn baryon_sim_apply_impulse(
		sim: BaryonSimulatorPtr,
		body_id: u32,
		ix: f32,
		iy: f32,
		iz: f32,
	) -> bool;
	pub fn baryon_sim_apply_torque(
		sim: BaryonSimulatorPtr,
		body_id: u32,
		tx: f32,
		ty: f32,
		tz: f32,
	) -> bool;

	pub fn baryon_sim_create_distance_constraint(
		sim: BaryonSimulatorPtr,
		body_a: u32,
		body_b: u32,
		distance: f32,
	) -> u32;

	pub fn baryon_sim_create_revolute_constraint(
		sim: BaryonSimulatorPtr,
		body_a: u32,
		body_b: u32,
		anchor_a_x: f32,
		anchor_a_y: f32,
		anchor_a_z: f32,
		anchor_b_x: f32,
		anchor_b_y: f32,
		anchor_b_z: f32,
		axis_a_x: f32,
		axis_a_y: f32,
		axis_a_z: f32,
		axis_b_x: f32,
		axis_b_y: f32,
		axis_b_z: f32,
	) -> u32;

	pub fn baryon_sim_raycast(
		sim: BaryonSimulatorPtr,
		ox: f32,
		oy: f32,
		oz: f32,
		dx: f32,
		dy: f32,
		dz: f32,
		max_dist: f32,
		out_body_id: *mut u32,
		out_hit_pos: *mut f32,
		out_hit_normal: *mut f32,
		out_hit_dist: *mut f32,
	) -> bool;
}
