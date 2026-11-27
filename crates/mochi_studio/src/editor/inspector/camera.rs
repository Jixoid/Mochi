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
	insert_vec4_field,
};


pub fn build_camera_section(
	tree: &mut UiTree,
	parent: ObjectID,
	entity: &EntityDesc,
	state: &Arc<Mutex<EditorState>>,
	tasks: &TaskQueue,
) {
	let Some(cam) = &entity.camera else { return };
	let sec = insert_section(tree, parent, "Kamera", true);

	insert_float_field(
		tree,
		sec,
		"FOV",
		cam.fov_deg,
		10.0,
		160.0,
		1.0,
		"°",
		Arc::clone(state),
		tasks.clone(),
		|e, _idx, _vp, val| {
			if let Some(c) = &mut e.camera {
				c.fov_deg = val.clamp(10.0, 160.0);
			}
		},
	);

	insert_float_field(
		tree,
		sec,
		"Near Clip",
		cam.near,
		0.001,
		100.0,
		0.05,
		"",
		Arc::clone(state),
		tasks.clone(),
		|e, _idx, _vp, val| {
			if let Some(c) = &mut e.camera {
				c.near = val.max(0.001);
			}
		},
	);

	insert_float_field(
		tree,
		sec,
		"Far Clip",
		cam.far,
		1.0,
		100_000.0,
		10.0,
		"",
		Arc::clone(state),
		tasks.clone(),
		|e, _idx, _vp, val| {
			if let Some(c) = &mut e.camera {
				c.far = val.max(1.0);
			}
		},
	);

	insert_vec4_field(
		tree,
		sec,
		"Clear Color (RGBA)",
		cam.clear_color,
		[0.1, 0.15, 0.25, 1.0],
		0.02,
		Some(0.0),
		Some(1.0),
		Arc::clone(state),
		tasks.clone(),
		|e, _idx, _vp, val| {
			if let Some(c) = &mut e.camera {
				c.clear_color = val;
			}
		},
	);

	insert_switch_field(
		tree,
		sec,
		"Aktif Kamera",
		cam.is_active,
		Arc::clone(state),
		tasks.clone(),
		|e, _idx, _vp, checked| {
			if let Some(c) = &mut e.camera {
				c.is_active = checked;
			}
		},
	);

	insert_remove_component_button(
		tree,
		sec,
		"Kamera",
		false,
		Arc::clone(state),
		tasks.clone(),
		|e| e.camera = None,
	);
}
