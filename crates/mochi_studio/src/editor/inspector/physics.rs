/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};

use mochi_asset::EntityDesc;
use nimbus::core::{ObjectID, TaskQueue, UiTree};

use super::super::state::EditorState;
use super::fields::{
	insert_combo_field, insert_edit_field, insert_float_field, insert_remove_component_button,
	insert_section, insert_switch_field, insert_vec3_field,
};


pub fn build_physics_sections(
	tree: &mut UiTree,
	parent: ObjectID,
	entity: &EntityDesc,
	state: &Arc<Mutex<EditorState>>,
	tasks: &TaskQueue,
) {
	if let Some(rb) = &entity.rigid_body {
		let sec = insert_section(tree, parent, "RigidBody (Fizik)", true);

		let body_types = ["dynamic", "static", "kinematic"];
		let selected_bt = body_types
			.iter()
			.position(|t| t.eq_ignore_ascii_case(&rb.body_type))
			.unwrap_or(0);
		insert_combo_field(
			tree,
			sec,
			"Gövde Tipi",
			&body_types,
			selected_bt,
			false,
			Arc::clone(state),
			tasks.clone(),
			|e, _idx, _vp, label| {
				if let Some(r) = &mut e.rigid_body {
					r.body_type = label;
				}
			},
		);

		insert_float_field(
			tree,
			sec,
			"Kütle",
			rb.mass,
			0.0,
			10_000.0,
			0.5,
			" kg",
			Arc::clone(state),
			tasks.clone(),
			|e, _idx, _vp, val| {
				if let Some(r) = &mut e.rigid_body {
					r.mass = val.max(0.0);
				}
			},
		);

		insert_float_field(
			tree,
			sec,
			"Linear Damping",
			rb.linear_damping,
			0.0,
			10.0,
			0.02,
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, _idx, _vp, val| {
				if let Some(r) = &mut e.rigid_body {
					r.linear_damping = val.max(0.0);
				}
			},
		);

		insert_float_field(
			tree,
			sec,
			"Angular Damping",
			rb.angular_damping,
			0.0,
			10.0,
			0.02,
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, _idx, _vp, val| {
				if let Some(r) = &mut e.rigid_body {
					r.angular_damping = val.max(0.0);
				}
			},
		);

		insert_float_field(
			tree,
			sec,
			"Yerçekimi Çarpanı",
			rb.gravity_scale,
			-10.0,
			10.0,
			0.1,
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, _idx, _vp, val| {
				if let Some(r) = &mut e.rigid_body {
					r.gravity_scale = val;
				}
			},
		);

		insert_switch_field(
			tree,
			sec,
			"Sürekli Çarpışma (CCD)",
			rb.use_ccd,
			Arc::clone(state),
			tasks.clone(),
			|e, _idx, _vp, checked| {
				if let Some(r) = &mut e.rigid_body {
					r.use_ccd = checked;
				}
			},
		);

		insert_remove_component_button(
			tree,
			sec,
			"RigidBody",
			false,
			Arc::clone(state),
			tasks.clone(),
			|e| e.rigid_body = None,
		);
	}

	if let Some(col) = &entity.collider {
		let sec = insert_section(tree, parent, "Collider", true);

		let shapes = ["box", "sphere", "capsule"];
		let selected_sh = shapes
			.iter()
			.position(|s| s.eq_ignore_ascii_case(&col.shape))
			.unwrap_or(0);
		insert_combo_field(
			tree,
			sec,
			"Şekil",
			&shapes,
			selected_sh,
			true,
			Arc::clone(state),
			tasks.clone(),
			|e, _idx, _vp, label| {
				if let Some(c) = &mut e.collider {
					c.shape = label;
				}
			},
		);

		match col.shape.to_lowercase().as_str() {
			"sphere" => {
				insert_float_field(
					tree,
					sec,
					"Yarıçap",
					col.radius.unwrap_or(0.5),
					0.01,
					1000.0,
					0.05,
					" m",
					Arc::clone(state),
					tasks.clone(),
					|e, _idx, _vp, val| {
						if let Some(c) = &mut e.collider {
							c.radius = Some(val.max(0.01));
						}
					},
				);
			}
			"capsule" => {
				insert_float_field(
					tree,
					sec,
					"Yarıçap",
					col.radius.unwrap_or(0.5),
					0.01,
					1000.0,
					0.05,
					" m",
					Arc::clone(state),
					tasks.clone(),
					|e, _idx, _vp, val| {
						if let Some(c) = &mut e.collider {
							c.radius = Some(val.max(0.01));
						}
					},
				);
				insert_float_field(
					tree,
					sec,
					"Yükseklik",
					col.height.unwrap_or(1.0),
					0.01,
					1000.0,
					0.05,
					" m",
					Arc::clone(state),
					tasks.clone(),
					|e, _idx, _vp, val| {
						if let Some(c) = &mut e.collider {
							c.height = Some(val.max(0.01));
						}
					},
				);
			}
			_ => {
				insert_vec3_field(
					tree,
					sec,
					"Yarı Boyutlar (Half Extents)",
					col.half_extents.unwrap_or([0.5, 0.5, 0.5]),
					[0.5, 0.5, 0.5],
					0.05,
					Some(0.01),
					None,
					"",
					Arc::clone(state),
					tasks.clone(),
					|e, _idx, _vp, val| {
						if let Some(c) = &mut e.collider {
							c.half_extents = Some([val[0].max(0.01), val[1].max(0.01), val[2].max(0.01)]);
						}
					},
				);
			}
		}

		insert_float_field(
			tree,
			sec,
			"Sürtünme",
			col.friction,
			0.0,
			2.0,
			0.05,
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, _idx, _vp, val| {
				if let Some(c) = &mut e.collider {
					c.friction = val.clamp(0.0, 2.0);
				}
			},
		);

		insert_float_field(
			tree,
			sec,
			"Sekme (Restitution)",
			col.restitution,
			0.0,
			1.0,
			0.05,
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, _idx, _vp, val| {
				if let Some(c) = &mut e.collider {
					c.restitution = val.clamp(0.0, 1.0);
				}
			},
		);

		insert_remove_component_button(
			tree,
			sec,
			"Collider",
			false,
			Arc::clone(state),
			tasks.clone(),
			|e| e.collider = None,
		);
	}

	if let Some(sj) = &entity.spring_joint {
		let sec = insert_section(tree, parent, "Spring Joint", true);

		insert_edit_field(
			tree,
			sec,
			"Hedef Entity",
			sj.target_entity.clone(),
			Arc::clone(state),
			tasks.clone(),
			|e, _idx, _vp, val| {
				if let Some(j) = &mut e.spring_joint {
					j.target_entity = val;
				}
			},
		);

		insert_vec3_field(
			tree,
			sec,
			"Local Anchor A",
			sj.local_anchor_a,
			[0.0, 0.0, 0.0],
			0.05,
			None,
			None,
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, _idx, _vp, val| {
				if let Some(j) = &mut e.spring_joint {
					j.local_anchor_a = val;
				}
			},
		);

		insert_vec3_field(
			tree,
			sec,
			"Local Anchor B",
			sj.local_anchor_b,
			[0.0, 0.0, 0.0],
			0.05,
			None,
			None,
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, _idx, _vp, val| {
				if let Some(j) = &mut e.spring_joint {
					j.local_anchor_b = val;
				}
			},
		);

		insert_float_field(
			tree,
			sec,
			"Dinlenme Uzunluğu",
			sj.rest_length,
			0.0,
			1000.0,
			0.1,
			" m",
			Arc::clone(state),
			tasks.clone(),
			|e, _idx, _vp, val| {
				if let Some(j) = &mut e.spring_joint {
					j.rest_length = val.max(0.0);
				}
			},
		);

		insert_float_field(
			tree,
			sec,
			"Sertlik (Stiffness)",
			sj.stiffness,
			0.0,
			100_000.0,
			5.0,
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, _idx, _vp, val| {
				if let Some(j) = &mut e.spring_joint {
					j.stiffness = val.max(0.0);
				}
			},
		);

		insert_float_field(
			tree,
			sec,
			"Sönümleme (Damping)",
			sj.damping,
			0.0,
			10_000.0,
			0.5,
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, _idx, _vp, val| {
				if let Some(j) = &mut e.spring_joint {
					j.damping = val.max(0.0);
				}
			},
		);

		insert_remove_component_button(
			tree,
			sec,
			"Spring Joint",
			false,
			Arc::clone(state),
			tasks.clone(),
			|e| e.spring_joint = None,
		);
	}
}
