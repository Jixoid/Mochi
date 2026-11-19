#include "baryon_c_api.h"
#include "Baryon/Simulator.hpp"
#include "Baryon/Core/CoreComponents.hpp"
#include <memory>
#include <vector>

struct BaryonSimulatorOpaque {
	Baryon::Simulator simulator;
	Baryon::Vector3 current_gravity{0.0f, -9.81f, 0.0f};

	BaryonSimulatorOpaque(float gx, float gy, float gz, float fixed_dt) {
		current_gravity = {gx, gy, gz};
		simulator.setAccelerationField(current_gravity);
		simulator.setFixedTimeStep(fixed_dt > 0.0f ? fixed_dt : (1.0f / 60.0f));
	}
};

extern "C" {

BaryonSimulator* baryon_sim_create(float gravity_x, float gravity_y, float gravity_z, float fixed_dt) {
	try {
		return new BaryonSimulatorOpaque(gravity_x, gravity_y, gravity_z, fixed_dt);
	} catch (...) {
		return nullptr;
	}
}

void baryon_sim_destroy(BaryonSimulator* sim) {
	delete sim;
}

void baryon_sim_step(BaryonSimulator* sim, float dt) {
	if (sim) {
		sim->simulator.step(dt);
	}
}

void baryon_sim_set_gravity(BaryonSimulator* sim, float gx, float gy, float gz) {
	if (sim) {
		sim->current_gravity = {gx, gy, gz};
		sim->simulator.setAccelerationField(sim->current_gravity);
	}
}

void baryon_sim_get_gravity(BaryonSimulator* sim, float* out_gx, float* out_gy, float* out_gz) {
	if (sim && out_gx && out_gy && out_gz) {
		*out_gx = sim->current_gravity.x;
		*out_gy = sim->current_gravity.y;
		*out_gz = sim->current_gravity.z;
	}
}

static uint32_t init_created_body(
	BaryonSimulator* sim,
	Baryon::Body& rb,
	int body_type,
	float mass,
	float friction,
	float restitution,
	bool use_ccd
) {
	(void)rb.setBodyType(static_cast<Baryon::Core::BodyType>(body_type));
	(void)rb.setMass(mass);
	(void)rb.setCCD(use_ccd);

	auto entity = rb.getEntity();
	if (sim->simulator.getRegistry().hasComponent<Baryon::Core::Material>(entity)) {
		auto& mat = sim->simulator.getRegistry().getComponent<Baryon::Core::Material>(entity);
		mat.friction = friction;
		mat.restitution = restitution;
	}

	return entity.id;
}

uint32_t baryon_sim_create_body_box(
	BaryonSimulator* sim,
	float px, float py, float pz,
	float qx, float qy, float qz, float qw,
	float hx, float hy, float hz,
	int body_type, float mass,
	float friction, float restitution,
	bool use_ccd
) {
	if (!sim) return 0xFFFFFFFF;

	Baryon::Pose pose(Baryon::Vector3(px, py, pz), Baryon::Quaternion(qx, qy, qz, qw));
	Baryon::collision::CollisionShape shape{Baryon::collision::BoxShape{Baryon::Vector3{hx, hy, hz}}};
	Baryon::Body rb = sim->simulator.createBody(pose, shape);

	return init_created_body(sim, rb, body_type, mass, friction, restitution, use_ccd);
}

uint32_t baryon_sim_create_body_sphere(
	BaryonSimulator* sim,
	float px, float py, float pz,
	float qx, float qy, float qz, float qw,
	float radius,
	int body_type, float mass,
	float friction, float restitution,
	bool use_ccd
) {
	if (!sim) return 0xFFFFFFFF;

	Baryon::Pose pose(Baryon::Vector3(px, py, pz), Baryon::Quaternion(qx, qy, qz, qw));
	Baryon::collision::CollisionShape shape{Baryon::collision::SphereShape{radius}};
	Baryon::Body rb = sim->simulator.createBody(pose, shape);

	return init_created_body(sim, rb, body_type, mass, friction, restitution, use_ccd);
}

uint32_t baryon_sim_create_body_capsule(
	BaryonSimulator* sim,
	float px, float py, float pz,
	float qx, float qy, float qz, float qw,
	float radius, float height,
	int body_type, float mass,
	float friction, float restitution,
	bool use_ccd
) {
	if (!sim) return 0xFFFFFFFF;

	Baryon::Pose pose(Baryon::Vector3(px, py, pz), Baryon::Quaternion(qx, qy, qz, qw));
	Baryon::collision::CollisionShape shape{Baryon::collision::CapsuleShape{radius, height}};
	Baryon::Body rb = sim->simulator.createBody(pose, shape);

	return init_created_body(sim, rb, body_type, mass, friction, restitution, use_ccd);
}

bool baryon_sim_destroy_body(BaryonSimulator* sim, uint32_t body_id) {
	if (!sim) return false;
	Baryon::ecs::Entity entity{body_id};
	if (!sim->simulator.getRegistry().isAlive(entity)) return false;

	Baryon::Body rb(entity, sim->simulator.getRegistry());
	sim->simulator.destroyBody(rb);
	return true;
}

bool baryon_sim_is_body_alive(BaryonSimulator* sim, uint32_t body_id) {
	if (!sim) return false;
	return sim->simulator.getRegistry().isAlive(Baryon::ecs::Entity{body_id});
}

bool baryon_sim_set_collider_box(
	BaryonSimulator* sim, uint32_t body_id,
	float hx, float hy, float hz,
	float friction, float restitution
) {
	if (!sim) return false;
	Baryon::ecs::Entity entity{body_id};
	if (!sim->simulator.getRegistry().isAlive(entity)) return false;

	if (sim->simulator.getRegistry().hasComponent<Baryon::ecs::ColliderData>(entity)) {
		auto& col = sim->simulator.getRegistry().getComponent<Baryon::ecs::ColliderData>(entity);
		col.shape = Baryon::collision::CollisionShape{Baryon::collision::BoxShape{Baryon::Vector3{hx, hy, hz}}};
	}
	if (sim->simulator.getRegistry().hasComponent<Baryon::Core::Material>(entity)) {
		auto& mat = sim->simulator.getRegistry().getComponent<Baryon::Core::Material>(entity);
		mat.friction = friction;
		mat.restitution = restitution;
	}
	if (sim->simulator.getRegistry().hasComponent<Baryon::Core::MassProps>(entity)) {
		auto& massProps = sim->simulator.getRegistry().getComponent<Baryon::Core::MassProps>(entity);
		float m = massProps.mass;
		if (m > 0.0f) {
			float sx = hx * 2.0f;
			float sy = hy * 2.0f;
			float sz = hz * 2.0f;
			Baryon::Matrix3x3 I = Baryon::Matrix3x3::identity();
			I[0].x = (1.0f / 12.0f) * m * (sy * sy + sz * sz);
			I[1].y = (1.0f / 12.0f) * m * (sx * sx + sz * sz);
			I[2].z = (1.0f / 12.0f) * m * (sx * sx + sy * sy);
			Baryon::Body rb(entity, sim->simulator.getRegistry());
			(void)rb.setLocalInertiaTensor(I);
		}
	}
	return true;
}

bool baryon_sim_set_collider_sphere(
	BaryonSimulator* sim, uint32_t body_id,
	float radius,
	float friction, float restitution
) {
	if (!sim) return false;
	Baryon::ecs::Entity entity{body_id};
	if (!sim->simulator.getRegistry().isAlive(entity)) return false;

	if (sim->simulator.getRegistry().hasComponent<Baryon::ecs::ColliderData>(entity)) {
		auto& col = sim->simulator.getRegistry().getComponent<Baryon::ecs::ColliderData>(entity);
		col.shape = Baryon::collision::CollisionShape{Baryon::collision::SphereShape{radius}};
	}
	if (sim->simulator.getRegistry().hasComponent<Baryon::Core::Material>(entity)) {
		auto& mat = sim->simulator.getRegistry().getComponent<Baryon::Core::Material>(entity);
		mat.friction = friction;
		mat.restitution = restitution;
	}
	return true;
}

bool baryon_sim_set_collider_capsule(
	BaryonSimulator* sim, uint32_t body_id,
	float radius, float height,
	float friction, float restitution
) {
	if (!sim) return false;
	Baryon::ecs::Entity entity{body_id};
	if (!sim->simulator.getRegistry().isAlive(entity)) return false;

	if (sim->simulator.getRegistry().hasComponent<Baryon::ecs::ColliderData>(entity)) {
		auto& col = sim->simulator.getRegistry().getComponent<Baryon::ecs::ColliderData>(entity);
		col.shape = Baryon::collision::CollisionShape{Baryon::collision::CapsuleShape{radius, height}};
	}
	if (sim->simulator.getRegistry().hasComponent<Baryon::Core::Material>(entity)) {
		auto& mat = sim->simulator.getRegistry().getComponent<Baryon::Core::Material>(entity);
		mat.friction = friction;
		mat.restitution = restitution;
	}
	return true;
}

bool baryon_sim_get_transform(BaryonSimulator* sim, uint32_t body_id, float out_pos[3], float out_rot[4]) {
	if (!sim || !out_pos || !out_rot) return false;
	Baryon::ecs::Entity entity{body_id};
	if (!sim->simulator.getRegistry().isAlive(entity)) return false;

	Baryon::Body rb(entity, sim->simulator.getRegistry());
	auto pos_res = rb.getPosition();
	auto rot_res = rb.getOrientation();
	if (!pos_res.has_value() || !rot_res.has_value()) return false;

	out_pos[0] = pos_res.value().x;
	out_pos[1] = pos_res.value().y;
	out_pos[2] = pos_res.value().z;

	out_rot[0] = rot_res.value().x;
	out_rot[1] = rot_res.value().y;
	out_rot[2] = rot_res.value().z;
	out_rot[3] = rot_res.value().w;
	return true;
}

bool baryon_sim_set_transform(
	BaryonSimulator* sim, uint32_t body_id,
	float px, float py, float pz,
	float qx, float qy, float qz, float qw
) {
	if (!sim) return false;
	Baryon::ecs::Entity entity{body_id};
	if (!sim->simulator.getRegistry().isAlive(entity)) return false;

	Baryon::Body rb(entity, sim->simulator.getRegistry());
	(void)rb.setPosition(Baryon::Vector3{px, py, pz});
	if (sim->simulator.getRegistry().hasComponent<Baryon::Pose>(entity)) {
		auto& pose = sim->simulator.getRegistry().getComponent<Baryon::Pose>(entity);
		pose.orientation = Baryon::Quaternion{qx, qy, qz, qw};
	}
	return true;
}

bool baryon_sim_get_velocity(BaryonSimulator* sim, uint32_t body_id, float out_lin[3], float out_ang[3]) {
	if (!sim || !out_lin || !out_ang) return false;
	Baryon::ecs::Entity entity{body_id};
	if (!sim->simulator.getRegistry().isAlive(entity)) return false;

	if (sim->simulator.getRegistry().hasComponent<Baryon::Core::Motion>(entity)) {
		const auto& motion = sim->simulator.getRegistry().getComponent<Baryon::Core::Motion>(entity);
		out_lin[0] = motion.linearVelocity.x;
		out_lin[1] = motion.linearVelocity.y;
		out_lin[2] = motion.linearVelocity.z;

		out_ang[0] = motion.angularVelocity.x;
		out_ang[1] = motion.angularVelocity.y;
		out_ang[2] = motion.angularVelocity.z;
		return true;
	}
	return false;
}

bool baryon_sim_set_velocity(
	BaryonSimulator* sim, uint32_t body_id,
	float lx, float ly, float lz,
	float ax, float ay, float az
) {
	if (!sim) return false;
	Baryon::ecs::Entity entity{body_id};
	if (!sim->simulator.getRegistry().isAlive(entity)) return false;

	Baryon::Body rb(entity, sim->simulator.getRegistry());
	(void)rb.setLinearVelocity(Baryon::Vector3{lx, ly, lz});
	if (sim->simulator.getRegistry().hasComponent<Baryon::Core::Motion>(entity)) {
		auto& motion = sim->simulator.getRegistry().getComponent<Baryon::Core::Motion>(entity);
		motion.angularVelocity = Baryon::Vector3{ax, ay, az};
	}
	return true;
}

bool baryon_sim_apply_force(BaryonSimulator* sim, uint32_t body_id, float fx, float fy, float fz) {
	if (!sim) return false;
	Baryon::ecs::Entity entity{body_id};
	if (!sim->simulator.getRegistry().isAlive(entity)) return false;

	if (sim->simulator.getRegistry().hasComponent<Baryon::Core::Motion>(entity)) {
		auto& motion = sim->simulator.getRegistry().getComponent<Baryon::Core::Motion>(entity);
		motion.externalForce += Baryon::Vector3{fx, fy, fz};

		// Wake up body
		if (sim->simulator.getRegistry().hasComponent<Baryon::Core::BodyState>(entity)) {
			auto& state = sim->simulator.getRegistry().getComponent<Baryon::Core::BodyState>(entity);
			state.isSleeping = false;
			state.sleepTimer = 0.0f;
		}
		return true;
	}
	return false;
}

bool baryon_sim_apply_impulse(BaryonSimulator* sim, uint32_t body_id, float ix, float iy, float iz) {
	if (!sim) return false;
	Baryon::ecs::Entity entity{body_id};
	if (!sim->simulator.getRegistry().isAlive(entity)) return false;

	if (sim->simulator.getRegistry().hasComponent<Baryon::Core::Motion>(entity)
		&& sim->simulator.getRegistry().hasComponent<Baryon::Core::MassProps>(entity)) {
		auto& motion = sim->simulator.getRegistry().getComponent<Baryon::Core::Motion>(entity);
		const auto& mass = sim->simulator.getRegistry().getComponent<Baryon::Core::MassProps>(entity);
		motion.linearVelocity += Baryon::Vector3{ix, iy, iz} * mass.inverseMass;

		// Wake up body
		if (sim->simulator.getRegistry().hasComponent<Baryon::Core::BodyState>(entity)) {
			auto& state = sim->simulator.getRegistry().getComponent<Baryon::Core::BodyState>(entity);
			state.isSleeping = false;
			state.sleepTimer = 0.0f;
		}
		return true;
	}
	return false;
}

bool baryon_sim_apply_torque(BaryonSimulator* sim, uint32_t body_id, float tx, float ty, float tz) {
	if (!sim) return false;
	Baryon::ecs::Entity entity{body_id};
	if (!sim->simulator.getRegistry().isAlive(entity)) return false;

	if (sim->simulator.getRegistry().hasComponent<Baryon::Core::Motion>(entity)) {
		auto& motion = sim->simulator.getRegistry().getComponent<Baryon::Core::Motion>(entity);
		motion.externalTorque += Baryon::Vector3{tx, ty, tz};
		return true;
	}
	return false;
}

uint32_t baryon_sim_create_distance_constraint(
	BaryonSimulator* sim,
	uint32_t body_a, uint32_t body_b,
	float distance
) {
	if (!sim) return 0xFFFFFFFF;
	Baryon::ecs::Entity eA{body_a};
	Baryon::ecs::Entity eB{body_b};
	if (!sim->simulator.getRegistry().isAlive(eA) || !sim->simulator.getRegistry().isAlive(eB)) return 0xFFFFFFFF;

	Baryon::Body rbA(eA, sim->simulator.getRegistry());
	Baryon::Body rbB(eB, sim->simulator.getRegistry());
	sim->simulator.createDistanceConstraint(rbA, rbB, distance);
	return eA.id;
}

uint32_t baryon_sim_create_revolute_constraint(
	BaryonSimulator* sim,
	uint32_t body_a, uint32_t body_b,
	float anchor_a_x, float anchor_a_y, float anchor_a_z,
	float anchor_b_x, float anchor_b_y, float anchor_b_z,
	float axis_a_x, float axis_a_y, float axis_a_z,
	float axis_b_x, float axis_b_y, float axis_b_z
) {
	if (!sim) return 0xFFFFFFFF;
	Baryon::ecs::Entity eA{body_a};
	Baryon::ecs::Entity eB{body_b};
	if (!sim->simulator.getRegistry().isAlive(eA) || !sim->simulator.getRegistry().isAlive(eB)) return 0xFFFFFFFF;

	Baryon::Body rbA(eA, sim->simulator.getRegistry());
	Baryon::Body rbB(eB, sim->simulator.getRegistry());
	sim->simulator.createRevoluteConstraint(
		rbA, rbB,
		Baryon::Vector3{anchor_a_x, anchor_a_y, anchor_a_z},
		Baryon::Vector3{anchor_b_x, anchor_b_y, anchor_b_z},
		Baryon::Vector3{axis_a_x, axis_a_y, axis_a_z},
		Baryon::Vector3{axis_b_x, axis_b_y, axis_b_z}
	);
	return eA.id;
}

bool baryon_sim_raycast(
	BaryonSimulator* sim,
	float ox, float oy, float oz,
	float dx, float dy, float dz,
	float max_dist,
	uint32_t* out_body_id,
	float out_hit_pos[3],
	float out_hit_normal[3],
	float* out_hit_dist
) {
	if (!sim) return false;
	auto hit = sim->simulator.raycast(
		Baryon::Vector3{ox, oy, oz},
		Baryon::Vector3{dx, dy, dz},
		max_dist
	);
	if (hit.hasHit) {
		if (out_body_id) *out_body_id = hit.entity.id;
		if (out_hit_pos) {
			out_hit_pos[0] = hit.position.x;
			out_hit_pos[1] = hit.position.y;
			out_hit_pos[2] = hit.position.z;
		}
		if (out_hit_normal) {
			out_hit_normal[0] = hit.normal.x;
			out_hit_normal[1] = hit.normal.y;
			out_hit_normal[2] = hit.normal.z;
		}
		if (out_hit_dist) *out_hit_dist = hit.distance;
		return true;
	}
	return false;
}

} // extern "C"
