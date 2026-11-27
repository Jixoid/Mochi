/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};

use mochi_asset::{EntityDesc, TransformDesc};
use nimbus::core::{Anchor, ObjectID, TaskQueue, UiTree};
use nimbus::ui::{Button, ButtonVariant};

use super::super::state::EditorState;
use super::super::viewport::{sync_gizmo_from_selection, update_header_and_status};
use super::fields::{euler_deg_to_quat, insert_section, insert_vec3_field, quat_to_euler_deg};
use super::rebuild_inspector;


pub fn build_transform_section(
	tree: &mut UiTree,
	parent: ObjectID,
	entity: &EntityDesc,
	state: &Arc<Mutex<EditorState>>,
	tasks: &TaskQueue,
) {
	let transform = entity.transform.clone().unwrap_or_default();
	let euler_deg = quat_to_euler_deg(transform.rotation);
	let sec = insert_section(tree, parent, "Transform", true);

	insert_vec3_field(
		tree,
		sec,
		"Konum",
		transform.translation,
		[0.0, 0.0, 0.0],
		0.1,
		None,
		None,
		"",
		Arc::clone(state),
		tasks.clone(),
		|e, e_idx, vp, val| {
			let tf = e.transform.get_or_insert_with(TransformDesc::default);
			tf.translation = val;
			vp.update_transform(e_idx, tf.clone());
		},
	);

	insert_vec3_field(
		tree,
		sec,
		"Dönüş (°)",
		euler_deg,
		[0.0, 0.0, 0.0],
		1.0,
		None,
		None,
		"°",
		Arc::clone(state),
		tasks.clone(),
		|e, e_idx, vp, val| {
			let tf = e.transform.get_or_insert_with(TransformDesc::default);
			tf.rotation = euler_deg_to_quat(val);
			vp.update_transform(e_idx, tf.clone());
		},
	);

	insert_vec3_field(
		tree,
		sec,
		"Ölçek",
		transform.scale,
		[1.0, 1.0, 1.0],
		0.05,
		Some(0.01),
		None,
		"",
		Arc::clone(state),
		tasks.clone(),
		|e, e_idx, vp, val| {
			let tf = e.transform.get_or_insert_with(TransformDesc::default);
			tf.scale = [val[0].max(0.01), val[1].max(0.01), val[2].max(0.01)];
			vp.update_transform(e_idx, tf.clone());
		},
	);

	let state_reset = Arc::clone(state);
	let tasks_reset = tasks.clone();
	let reset_id = tree.insert_child(
		sec,
		Button::builder()
			.text("Transformu Sıfırla")
			.font_size(12.0)
			.variant(ButtonVariant::Outlined)
			.on_click(move || {
				{
					let mut guard = state_reset.lock().unwrap();
					let Some(idx) = guard.selected_index else {
						return;
					};
					guard.record_undo("Transform Sıfırla");
					let default_tf = TransformDesc::default();
					if let Some(e) = guard.scene.entities.get_mut(idx) {
						e.transform = Some(default_tf.clone());
						guard.dirty = true;
					}
					guard.vp_ctrl.update_transform(idx, default_tf);
				}
				let state_inner = Arc::clone(&state_reset);
				let tasks_inner = tasks_reset.clone();
				tasks_reset.push_tree(move |tree| {
					update_header_and_status(tree, &state_inner, Some("Transform sıfırlandı".to_string()));
					rebuild_inspector(tree, &state_inner, &tasks_inner);
					sync_gizmo_from_selection(tree, &state_inner);
				});
			})
			.build(),
	);
	tree.set_anchor(reset_id, Some(Anchor::TopFill));
}
