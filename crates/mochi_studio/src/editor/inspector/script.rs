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


pub fn build_script_section(
	tree: &mut UiTree,
	parent: ObjectID,
	entity: &EntityDesc,
	state: &Arc<Mutex<EditorState>>,
	tasks: &TaskQueue,
) {
	let Some(script) = &entity.script else { return };
	let sec = insert_section(tree, parent, "Script", true);

	insert_edit_field(
		tree,
		sec,
		"Script Yolu",
		script.path.clone(),
		Arc::clone(state),
		tasks.clone(),
		|e, _idx, _vp, val| {
			if let Some(s) = &mut e.script {
				s.path = val;
			}
		},
	);

	insert_switch_field(
		tree,
		sec,
		"Aktif",
		script.is_active,
		Arc::clone(state),
		tasks.clone(),
		|e, _idx, _vp, checked| {
			if let Some(s) = &mut e.script {
				s.is_active = checked;
			}
		},
	);

	insert_remove_component_button(
		tree,
		sec,
		"Script",
		false,
		Arc::clone(state),
		tasks.clone(),
		|e| e.script = None,
	);
}
