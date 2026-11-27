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
	insert_edit_field, insert_remove_component_button, insert_section, insert_switch_field,
};


pub fn build_mesh_section(
	tree: &mut UiTree,
	parent: ObjectID,
	entity: &EntityDesc,
	state: &Arc<Mutex<EditorState>>,
	tasks: &TaskQueue,
) {
	let Some(mr) = &entity.mesh_renderer else {
		return;
	};
	let sec = insert_section(tree, parent, "Mesh Renderer", true);

	insert_edit_field(
		tree,
		sec,
		"Mesh Varlığı",
		mr.asset.clone(),
		Arc::clone(state),
		tasks.clone(),
		|e, _idx, _vp, val| {
			if let Some(m) = &mut e.mesh_renderer {
				m.asset = val;
			}
		},
	);

	insert_switch_field(
		tree,
		sec,
		"Görünür",
		mr.is_visible,
		Arc::clone(state),
		tasks.clone(),
		|e, idx, vp, checked| {
			if let Some(m) = &mut e.mesh_renderer {
				m.is_visible = checked;
				vp.update_mesh_flags(idx, m.is_visible, m.cast_shadows);
			}
		},
	);

	insert_switch_field(
		tree,
		sec,
		"Gölge Düşür",
		mr.cast_shadows,
		Arc::clone(state),
		tasks.clone(),
		|e, idx, vp, checked| {
			if let Some(m) = &mut e.mesh_renderer {
				m.cast_shadows = checked;
				vp.update_mesh_flags(idx, m.is_visible, m.cast_shadows);
			}
		},
	);

	insert_remove_component_button(
		tree,
		sec,
		"Mesh Renderer",
		true,
		Arc::clone(state),
		tasks.clone(),
		|e| e.mesh_renderer = None,
	);
}
