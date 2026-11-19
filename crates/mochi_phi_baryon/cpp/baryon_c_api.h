#ifndef BARYON_C_API_H
#define BARYON_C_API_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct BaryonSimulatorOpaque BaryonSimulator;


// Simulator Lifecycle

BaryonSimulator* baryon_sim_create(float gravity_x, float gravity_y, float gravity_z, float fixed_dt);
void baryon_sim_destroy(BaryonSimulator* sim);
void baryon_sim_step(BaryonSimulator* sim, float dt);
void baryon_sim_set_gravity(BaryonSimulator* sim, float gx, float gy, float gz);
void baryon_sim_get_gravity(BaryonSimulator* sim, float* out_gx, float* out_gy, float* out_gz);


// Body Lifecycle & Shapes

uint32_t baryon_sim_create_body_box(
	BaryonSimulator* sim,
	float px, float py, float pz,
	float qx, float qy, float qz, float qw,
	float hx, float hy, float hz,
	int body_type, float mass,
	float friction, float restitution,
	bool use_ccd
);

uint32_t baryon_sim_create_body_sphere(
	BaryonSimulator* sim,
	float px, float py, float pz,
	float qx, float qy, float qz, float qw,
	float radius,
	int body_type, float mass,
	float friction, float restitution,
	bool use_ccd
);

uint32_t baryon_sim_create_body_capsule(
	BaryonSimulator* sim,
	float px, float py, float pz,
	float qx, float qy, float qz, float qw,
	float radius, float height,
	int body_type, float mass,
	float friction, float restitution,
	bool use_ccd
);

bool baryon_sim_destroy_body(BaryonSimulator* sim, uint32_t body_id);
bool baryon_sim_is_body_alive(BaryonSimulator* sim, uint32_t body_id);

bool baryon_sim_set_collider_box(
	BaryonSimulator* sim, uint32_t body_id,
	float hx, float hy, float hz,
	float friction, float restitution
);
bool baryon_sim_set_collider_sphere(
	BaryonSimulator* sim, uint32_t body_id,
	float radius,
	float friction, float restitution
);
bool baryon_sim_set_collider_capsule(
	BaryonSimulator* sim, uint32_t body_id,
	float radius, float height,
	float friction, float restitution
);

// Transform, Velocity & Dynamics

bool baryon_sim_get_transform(BaryonSimulator* sim, uint32_t body_id, float out_pos[3], float out_rot[4]);
bool baryon_sim_set_transform(
	BaryonSimulator* sim, uint32_t body_id,
	float px, float py, float pz,
	float qx, float qy, float qz, float qw
);

bool baryon_sim_get_velocity(BaryonSimulator* sim, uint32_t body_id, float out_lin[3], float out_ang[3]);
bool baryon_sim_set_velocity(
	BaryonSimulator* sim, uint32_t body_id,
	float lx, float ly, float lz,
	float ax, float ay, float az
);

bool baryon_sim_apply_force(BaryonSimulator* sim, uint32_t body_id, float fx, float fy, float fz);
bool baryon_sim_apply_impulse(BaryonSimulator* sim, uint32_t body_id, float ix, float iy, float iz);
bool baryon_sim_apply_torque(BaryonSimulator* sim, uint32_t body_id, float tx, float ty, float tz);

// Joints & Constraints

uint32_t baryon_sim_create_distance_constraint(
	BaryonSimulator* sim,
	uint32_t body_a, uint32_t body_b,
	float distance
);

uint32_t baryon_sim_create_revolute_constraint(
	BaryonSimulator* sim,
	uint32_t body_a, uint32_t body_b,
	float anchor_a_x, float anchor_a_y, float anchor_a_z,
	float anchor_b_x, float anchor_b_y, float anchor_b_z,
	float axis_a_x, float axis_a_y, float axis_a_z,
	float axis_b_x, float axis_b_y, float axis_b_z
);

// Raycast

bool baryon_sim_raycast(
	BaryonSimulator* sim,
	float ox, float oy, float oz,
	float dx, float dy, float dz,
	float max_dist,
	uint32_t* out_body_id,
	float out_hit_pos[3],
	float out_hit_normal[3],
	float* out_hit_dist
);

#ifdef __cplusplus
}
#endif

#endif // BARYON_C_API_H
