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
	insert_float_field, insert_remove_component_button, insert_section, insert_switch_field,
	insert_vec3_field,
};


pub fn build_lights_sections(
	tree: &mut UiTree,
	parent: ObjectID,
	entity: &EntityDesc,
	state: &Arc<Mutex<EditorState>>,
	tasks: &TaskQueue,
) {
	if let Some(light) = &entity.directional_light {
		let sec = insert_section(tree, parent, "Directional Light", true);

		insert_vec3_field(
			tree,
			sec,
			"Renk (RGB)",
			light.color,
			[1.0, 1.0, 1.0],
			0.02,
			Some(0.0),
			Some(1.0),
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, idx, vp, val| {
				if let Some(l) = &mut e.directional_light {
					l.color = val;
					vp.update_directional_light(idx, l.clone());
				}
			},
		);

		insert_float_field(
			tree,
			sec,
			"Yoğunluk",
			light.intensity,
			0.0,
			100.0,
			0.2,
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, idx, vp, val| {
				if let Some(l) = &mut e.directional_light {
					l.intensity = val.max(0.0);
					vp.update_directional_light(idx, l.clone());
				}
			},
		);

		insert_vec3_field(
			tree,
			sec,
			"Yön (Direction)",
			light.direction,
			[0.5, -1.0, 0.5],
			0.05,
			Some(-1.0),
			Some(1.0),
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, idx, vp, val| {
				if let Some(l) = &mut e.directional_light {
					l.direction = val;
					vp.update_directional_light(idx, l.clone());
				}
			},
		);

		insert_switch_field(
			tree,
			sec,
			"Gölge Düşür",
			light.cast_shadows,
			Arc::clone(state),
			tasks.clone(),
			|e, idx, vp, checked| {
				if let Some(l) = &mut e.directional_light {
					l.cast_shadows = checked;
					vp.update_directional_light(idx, l.clone());
				}
			},
		);

		insert_remove_component_button(
			tree,
			sec,
			"Directional Light",
			true,
			Arc::clone(state),
			tasks.clone(),
			|e| e.directional_light = None,
		);
	}

	if let Some(light) = &entity.point_light {
		let sec = insert_section(tree, parent, "Point Light", true);

		insert_vec3_field(
			tree,
			sec,
			"Renk (RGB)",
			light.color,
			[1.0, 1.0, 1.0],
			0.02,
			Some(0.0),
			Some(1.0),
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, idx, vp, val| {
				if let Some(l) = &mut e.point_light {
					l.color = val;
					vp.update_point_light(idx, l.clone());
				}
			},
		);

		insert_float_field(
			tree,
			sec,
			"Yoğunluk",
			light.intensity,
			0.0,
			500.0,
			0.2,
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, idx, vp, val| {
				if let Some(l) = &mut e.point_light {
					l.intensity = val.max(0.0);
					vp.update_point_light(idx, l.clone());
				}
			},
		);

		insert_float_field(
			tree,
			sec,
			"Yarıçap",
			light.radius,
			0.1,
			500.0,
			0.5,
			" m",
			Arc::clone(state),
			tasks.clone(),
			|e, idx, vp, val| {
				if let Some(l) = &mut e.point_light {
					l.radius = val.max(0.1);
					vp.update_point_light(idx, l.clone());
				}
			},
		);

		insert_remove_component_button(
			tree,
			sec,
			"Point Light",
			true,
			Arc::clone(state),
			tasks.clone(),
			|e| e.point_light = None,
		);
	}

	if let Some(light) = &entity.spot_light {
		let sec = insert_section(tree, parent, "Spot Light", true);

		insert_vec3_field(
			tree,
			sec,
			"Renk (RGB)",
			light.color,
			[1.0, 1.0, 1.0],
			0.02,
			Some(0.0),
			Some(1.0),
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, idx, vp, val| {
				if let Some(l) = &mut e.spot_light {
					l.color = val;
					vp.update_spot_light(idx, l.clone());
				}
			},
		);

		insert_float_field(
			tree,
			sec,
			"Yoğunluk",
			light.intensity,
			0.0,
			500.0,
			0.2,
			"",
			Arc::clone(state),
			tasks.clone(),
			|e, idx, vp, val| {
				if let Some(l) = &mut e.spot_light {
					l.intensity = val.max(0.0);
					vp.update_spot_light(idx, l.clone());
				}
			},
		);

		insert_float_field(
			tree,
			sec,
			"Menzil",
			light.radius,
			0.1,
			500.0,
			0.5,
			" m",
			Arc::clone(state),
			tasks.clone(),
			|e, idx, vp, val| {
				if let Some(l) = &mut e.spot_light {
					l.radius = val.max(0.1);
					vp.update_spot_light(idx, l.clone());
				}
			},
		);

		insert_float_field(
			tree,
			sec,
			"İç Koni Açısı",
			light.inner_cone_angle_deg,
			1.0,
			89.0,
			1.0,
			"°",
			Arc::clone(state),
			tasks.clone(),
			|e, idx, vp, val| {
				if let Some(l) = &mut e.spot_light {
					l.inner_cone_angle_deg = val.clamp(1.0, 89.0);
					if l.outer_cone_angle_deg < l.inner_cone_angle_deg {
						l.outer_cone_angle_deg = l.inner_cone_angle_deg;
					}
					vp.update_spot_light(idx, l.clone());
				}
			},
		);

		insert_float_field(
			tree,
			sec,
			"Dış Koni Açısı",
			light.outer_cone_angle_deg,
			1.0,
			89.0,
			1.0,
			"°",
			Arc::clone(state),
			tasks.clone(),
			|e, idx, vp, val| {
				if let Some(l) = &mut e.spot_light {
					l.outer_cone_angle_deg = val.clamp(l.inner_cone_angle_deg, 89.0);
					vp.update_spot_light(idx, l.clone());
				}
			},
		);

		insert_switch_field(
			tree,
			sec,
			"Gölge Düşür",
			light.cast_shadows,
			Arc::clone(state),
			tasks.clone(),
			|e, idx, vp, checked| {
				if let Some(l) = &mut e.spot_light {
					l.cast_shadows = checked;
					vp.update_spot_light(idx, l.clone());
				}
			},
		);

		insert_remove_component_button(
			tree,
			sec,
			"Spot Light",
			true,
			Arc::clone(state),
			tasks.clone(),
			|e| e.spot_light = None,
		);
	}
}
