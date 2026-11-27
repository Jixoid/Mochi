/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_math::{Quat, Vec3};
use mochi_rendering::{DebugDraw, DebugIconKind};
use mochi_scene::{
	CameraComponent, ColliderComponent, ColliderShape, DirectionalLightComponent, Entity,
	PointLightComponent, Scene, SpotLightComponent, TransformComponent,
};


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ViewportOverlayOptions {
	pub show_grid: bool,
	pub show_gizmos: bool,
	pub show_colliders: bool,
}

impl Default for ViewportOverlayOptions {
	fn default() -> Self {
		Self {
			show_grid: true,
			show_gizmos: true,
			show_colliders: false,
		}
	}
}


pub fn populate_scene_overlays(
	scene: &Scene,
	selected_entities: &[Entity],
	options: ViewportOverlayOptions,
	debug: &mut DebugDraw,
) {
	debug.draw_grid = options.show_grid;

	let is_selected = |e: Entity| selected_entities.contains(&e);

	if options.show_gizmos {
		// Cameras
		for (entity, (tf, cam)) in scene
			.world()
			.query::<(&TransformComponent, &CameraComponent)>()
			.iter()
		{
			let sel = is_selected(entity);
			let color = if sel {
				[0.80, 0.62, 1.0, 1.0]
			} else {
				[0.73, 0.53, 0.98, 0.85]
			};
			debug.billboard(tf.translation, if sel { 1.15 } else { 0.95 }, color, DebugIconKind::Camera);

			let mut preview_cam = *cam;
			preview_cam.far = if sel {
				cam.far.min(24.0)
			} else {
				cam.far.min(3.2)
			};
			let vp = preview_cam.projection_matrix(16.0 / 9.0) * preview_cam.view_matrix(&tf.0);
			let inv_vp = vp.inverse();
			let frustum_col = if sel {
				[0.80, 0.62, 1.0, 0.95]
			} else {
				[0.73, 0.53, 0.98, 0.32]
			};
			debug.wire_frustum(inv_vp, frustum_col);
		}

		// Directional Lights
		for (entity, (tf, light)) in scene
			.world()
			.query::<(&TransformComponent, &DirectionalLightComponent)>()
			.iter()
		{
			let sel = is_selected(entity);
			let tint = light_tint(light.color, if sel { 1.0 } else { 0.9 });
			debug.billboard(
				tf.translation,
				if sel { 1.18 } else { 0.98 },
				tint,
				DebugIconKind::DirectionalLight,
			);

			let dir = tf.rotation * light.direction;
			let ray_col = light_tint(light.color, if sel { 0.95 } else { 0.45 });
			debug.arrow(tf.translation, dir, 2.2, 0.35, ray_col);

			if sel {
				let (u, v) = ortho_basis(dir);
				for offset in [u * 0.6, -u * 0.6, v * 0.6, -v * 0.6] {
					debug.arrow(
						tf.translation + offset,
						dir,
						1.8,
						0.25,
						light_tint(light.color, 0.65),
					);
				}
			}
		}

		// Point Lights
		for (entity, (tf, light)) in scene
			.world()
			.query::<(&TransformComponent, &PointLightComponent)>()
			.iter()
		{
			let sel = is_selected(entity);
			let tint = light_tint(light.color, if sel { 1.0 } else { 0.9 });
			debug.billboard(
				tf.translation,
				if sel { 1.15 } else { 0.95 },
				tint,
				DebugIconKind::PointLight,
			);

			if sel {
				debug.wire_sphere(tf.translation, light.radius, light_tint(light.color, 0.80));
			}
		}

		// Spot Lights
		for (entity, (tf, light)) in scene
			.world()
			.query::<(&TransformComponent, &SpotLightComponent)>()
			.iter()
		{
			let sel = is_selected(entity);
			let tint = light_tint(light.color, if sel { 1.0 } else { 0.9 });
			debug.billboard(
				tf.translation,
				if sel { 1.15 } else { 0.95 },
				tint,
				DebugIconKind::SpotLight,
			);

			let dir = tf.rotation * Vec3::new(0.0, 0.0, -1.0);
			if sel {
				let len = light.radius.min(30.0);
				debug.wire_cone(
					tf.translation,
					dir,
					len,
					light.inner_cone_angle_deg * 0.5,
					20,
					light_tint(light.color, 0.92),
				);
				debug.wire_cone(
					tf.translation,
					dir,
					len,
					light.outer_cone_angle_deg * 0.5,
					24,
					light_tint(light.color, 0.45),
				);
			} else {
				debug.ray(tf.translation, dir, 1.5, light_tint(light.color, 0.40));
			}
		}
	}

	// Physics Colliders
	for (entity, (tf, col)) in scene
		.world()
		.query::<(&TransformComponent, &ColliderComponent)>()
		.iter()
	{
		let sel = is_selected(entity);
		if !options.show_colliders && !sel {
			continue;
		}

		let color = if sel {
			[0.32, 0.88, 0.48, 0.95]
		} else {
			[0.32, 0.82, 0.45, 0.42]
		};
		let world_center = tf.translation + tf.rotation * col.def.offset_position;
		let world_rot = normalize_quat(tf.rotation * col.def.offset_rotation);
		let max_scale = tf.scale.x.abs().max(tf.scale.y.abs()).max(tf.scale.z.abs()).max(0.01);

		match &col.def.shape {
			ColliderShape::Box { half_extents } => {
				let scaled = Vec3::new(
					half_extents.x * tf.scale.x.abs(),
					half_extents.y * tf.scale.y.abs(),
					half_extents.z * tf.scale.z.abs(),
				);
				debug.wire_box(world_center, scaled, world_rot, color);
			}
			ColliderShape::Sphere { radius } => {
				debug.wire_sphere(world_center, radius * max_scale, color);
			}
			ColliderShape::Capsule { radius, height } => {
				debug.wire_capsule(
					world_center,
					radius * tf.scale.x.abs().max(tf.scale.z.abs()).max(0.01),
					height * tf.scale.y.abs(),
					world_rot,
					color,
				);
			}
			ColliderShape::Cylinder { radius, height } => {
				debug.wire_cylinder(
					world_center,
					radius * tf.scale.x.abs().max(tf.scale.z.abs()).max(0.01),
					height * tf.scale.y.abs(),
					world_rot,
					color,
				);
			}
			_ => {}
		}
	}
}


fn light_tint(c: Vec3<f32>, alpha: f32) -> [f32; 4] {
	let max_c = c.x.max(c.y).max(c.z).max(0.001);
	let scale = if max_c > 1.0 { 1.0 / max_c } else { 1.0 };
	[
		(c.x * scale).clamp(0.15, 1.0),
		(c.y * scale).clamp(0.15, 1.0),
		(c.z * scale).clamp(0.15, 1.0),
		alpha,
	]
}

fn ortho_basis(dir: Vec3<f32>) -> (Vec3<f32>, Vec3<f32>) {
	let len_sq = dir.x * dir.x + dir.y * dir.y + dir.z * dir.z;
	let d = if len_sq > 1e-8 {
		dir / len_sq.sqrt()
	} else {
		Vec3::new(0.0, -1.0, 0.0)
	};
	let up = if d.y.abs() < 0.99 {
		Vec3::new(0.0, 1.0, 0.0)
	} else {
		Vec3::new(1.0, 0.0, 0.0)
	};
	let u = d.cross(&up);
	let u_len = (u.x * u.x + u.y * u.y + u.z * u.z).sqrt().max(1e-6);
	let u = u / u_len;
	let v = d.cross(&u);
	(u, v)
}

fn normalize_quat(q: Quat<f32>) -> Quat<f32> {
	let len = (q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w).sqrt();
	if len > 1e-6 {
		Quat::new(q.x / len, q.y / len, q.z / len, q.w / len)
	} else {
		Quat::IDENTITY
	}
}
