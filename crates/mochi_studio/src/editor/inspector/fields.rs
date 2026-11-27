/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};

use mochi_asset::EntityDesc;
use nimbus::core::{Anchor, ObjectID, PropertyValue, TaskQueue, UiTree};
use nimbus::math::Color;
use nimbus::theme::{DColor, MonetRole};
use nimbus::ui::{Button, ButtonVariant, Collapsible, ComboBox, Edit, FloatSpinBox, Switch, Text, VBox};
use nimbus::ext::inspector::{Vec3Edit, Vec4Edit};

use crate::viewport_renderer::ViewportController;

use super::super::console::push_console_log;
use super::super::hierarchy::rebuild_hierarchy;
use super::super::state::EditorState;
use super::super::viewport::{sync_gizmo_from_selection, update_header_and_status};
use super::rebuild_inspector;


pub fn insert_section(
	tree: &mut UiTree,
	parent: ObjectID,
	title: &str,
	expanded: bool,
) -> ObjectID {
	let col_id = tree.insert_child(
		parent,
		Collapsible::builder()
			.title(title)
			.expanded(expanded)
			.spacing(6.0)
			.paddings(8.0)
			.corner_radius(6.0)
			.build(),
	);
	tree.set_anchor(col_id, Some(Anchor::TopFill));

	let vbox_id = tree.insert_child(
		col_id,
		VBox::builder()
			.spacing(6.0)
			.color(Color::TRANSPARENT)
			.build(),
	);
	tree.set_anchor(vbox_id, Some(Anchor::TopFill));
	vbox_id
}


pub fn insert_label(tree: &mut UiTree, parent: ObjectID, label: &str) {
	tree.insert_child(
		parent,
		Text::builder()
			.text(label)
			.font_size(12.0)
			.color(DColor::from(MonetRole::OnSurfaceVariant))
			.build(),
	);
}


pub fn insert_remove_component_button<F>(
	tree: &mut UiTree,
	parent: ObjectID,
	component_name: &'static str,
	reload_viewport: bool,
	state: Arc<Mutex<EditorState>>,
	tasks: TaskQueue,
	remove_fn: F,
) where
	F: Fn(&mut EntityDesc) + Send + Sync + 'static,
{
	let btn_id = tree.insert_child(
		parent,
		Button::builder()
			.text(format!("{component_name} Kaldır"))
			.font_size(12.0)
			.variant(ButtonVariant::Outlined)
			.on_click(move || {
				{
					let mut guard = state.lock().unwrap();
					let Some(idx) = guard.selected_index else {
						return;
					};
					guard.record_undo(format!("{component_name} Kaldır"));
					if let Some(e) = guard.scene.entities.get_mut(idx) {
						remove_fn(e);
						guard.dirty = true;
					}
					if reload_viewport {
						guard.reload_viewport_scene();
					}
				}
				let state_inner = Arc::clone(&state);
				let tasks_inner = tasks.clone();
				tasks.push_tree(move |tree| {
					let msg = format!("{component_name} bileşeni kaldırıldı");
					push_console_log(tree, &state_inner, "ECS", &msg);
					update_header_and_status(tree, &state_inner, Some(msg));
					rebuild_hierarchy(tree, &state_inner);
					rebuild_inspector(tree, &state_inner, &tasks_inner);
					sync_gizmo_from_selection(tree, &state_inner);
				});
			})
			.build(),
	);
	tree.set_anchor(btn_id, Some(Anchor::TopFill));
}


#[allow(clippy::too_many_arguments)]
pub fn insert_float_field<F>(
	tree: &mut UiTree,
	parent: ObjectID,
	label: &str,
	initial: f32,
	min: f64,
	max: f64,
	step: f64,
	suffix: &str,
	state: Arc<Mutex<EditorState>>,
	tasks: TaskQueue,
	apply: F,
) where
	F: Fn(&mut EntityDesc, usize, &ViewportController, f32) + Send + Sync + 'static,
{
	let mut builder = FloatSpinBox::builder()
		.title(label)
		.value(initial as f64)
		.range(min, max)
		.step(step)
		.decimals(2)
		.on_changed(move |val: f64| {
			let val_f32 = val as f32;
			{
				let mut guard = state.lock().unwrap();
				if let Some(idx) = guard.selected_index {
					let vp_ctrl = guard.vp_ctrl.clone();
					if let Some(entity) = guard.scene.entities.get_mut(idx) {
						apply(entity, idx, &vp_ctrl, val_f32);
						guard.dirty = true;
					}
				}
			}
			let state_inner = Arc::clone(&state);
			tasks.push_tree(move |tree| {
				update_header_and_status(tree, &state_inner, None);
				sync_gizmo_from_selection(tree, &state_inner);
			});
		});

	if !suffix.is_empty() {
		builder = builder.suffix(suffix);
	}

	let spin_id = tree.insert_child(parent, builder.build());
	tree.set_anchor(spin_id, Some(Anchor::TopFill));
}


#[allow(clippy::too_many_arguments)]
pub fn insert_vec3_field<F>(
	tree: &mut UiTree,
	parent: ObjectID,
	label: &str,
	initial: [f32; 3],
	default_values: [f32; 3],
	step: f32,
	min: Option<f32>,
	max: Option<f32>,
	suffix: &str,
	state: Arc<Mutex<EditorState>>,
	tasks: TaskQueue,
	apply: F,
) where
	F: Fn(&mut EntityDesc, usize, &ViewportController, [f32; 3]) + Send + Sync + 'static,
{
	insert_label(tree, parent, label);

	let mut edit =
		Vec3Edit::new(initial)
			.with_step(step)
			.with_on_changed(move |args: &[PropertyValue]| {
				let (Some(x), Some(y), Some(z)) = (
					args.first().and_then(|v| v.as_f32()),
					args.get(1).and_then(|v| v.as_f32()),
					args.get(2).and_then(|v| v.as_f32()),
				) else {
					return;
				};
				{
					let mut guard = state.lock().unwrap();
					if let Some(idx) = guard.selected_index {
						let vp_ctrl = guard.vp_ctrl.clone();
						if let Some(entity) = guard.scene.entities.get_mut(idx) {
							apply(entity, idx, &vp_ctrl, [x, y, z]);
							guard.dirty = true;
						}
					}
				}
				let state_inner = Arc::clone(&state);
				tasks.push_tree(move |tree| {
					update_header_and_status(tree, &state_inner, None);
					sync_gizmo_from_selection(tree, &state_inner);
				});
			});
	edit.default_values = default_values;
	edit.min = min;
	edit.max = max;
	if !suffix.is_empty() {
		edit.suffix = suffix.to_string();
	}

	let id = tree.insert_child(parent, edit);
	tree.set_anchor(id, Some(Anchor::TopFill));
}


#[allow(clippy::too_many_arguments)]
pub fn insert_vec4_field<F>(
	tree: &mut UiTree,
	parent: ObjectID,
	label: &str,
	initial: [f32; 4],
	default_values: [f32; 4],
	step: f32,
	min: Option<f32>,
	max: Option<f32>,
	state: Arc<Mutex<EditorState>>,
	tasks: TaskQueue,
	apply: F,
) where
	F: Fn(&mut EntityDesc, usize, &ViewportController, [f32; 4]) + Send + Sync + 'static,
{
	insert_label(tree, parent, label);

	let mut edit =
		Vec4Edit::new(initial)
			.with_step(step)
			.with_on_changed(move |args: &[PropertyValue]| {
				let (Some(x), Some(y), Some(z), Some(w)) = (
					args.first().and_then(|v| v.as_f32()),
					args.get(1).and_then(|v| v.as_f32()),
					args.get(2).and_then(|v| v.as_f32()),
					args.get(3).and_then(|v| v.as_f32()),
				) else {
					return;
				};
				{
					let mut guard = state.lock().unwrap();
					if let Some(idx) = guard.selected_index {
						let vp_ctrl = guard.vp_ctrl.clone();
						if let Some(entity) = guard.scene.entities.get_mut(idx) {
							apply(entity, idx, &vp_ctrl, [x, y, z, w]);
							guard.dirty = true;
						}
					}
				}
				let state_inner = Arc::clone(&state);
				tasks.push_tree(move |tree| {
					update_header_and_status(tree, &state_inner, None);
				});
			});
	edit.default_values = default_values;
	edit.min = min;
	edit.max = max;

	let id = tree.insert_child(parent, edit);
	tree.set_anchor(id, Some(Anchor::TopFill));
}


pub fn insert_switch_field<F>(
	tree: &mut UiTree,
	parent: ObjectID,
	label: &str,
	checked: bool,
	state: Arc<Mutex<EditorState>>,
	tasks: TaskQueue,
	apply: F,
) where
	F: Fn(&mut EntityDesc, usize, &ViewportController, bool) + Send + Sync + 'static,
{
	tree.insert_child(
		parent,
		Switch::builder()
			.text(label)
			.checked(checked)
			.on_changed(move |val: bool| {
				{
					let mut guard = state.lock().unwrap();
					if let Some(idx) = guard.selected_index {
						let vp_ctrl = guard.vp_ctrl.clone();
						if let Some(entity) = guard.scene.entities.get_mut(idx) {
							apply(entity, idx, &vp_ctrl, val);
							guard.dirty = true;
						}
					}
				}
				let state_inner = Arc::clone(&state);
				tasks.push_tree(move |tree| {
					update_header_and_status(tree, &state_inner, None);
				});
			})
			.build(),
	);
}


pub fn insert_edit_field<F>(
	tree: &mut UiTree,
	parent: ObjectID,
	title: &str,
	initial: String,
	state: Arc<Mutex<EditorState>>,
	tasks: TaskQueue,
	apply: F,
) where
	F: Fn(&mut EntityDesc, usize, &ViewportController, String) + Send + Sync + 'static,
{
	let id = tree.insert_child(
		parent,
		Edit::builder()
			.title(title)
			.text(initial)
			.on_changed(move |val: String| {
				{
					let mut guard = state.lock().unwrap();
					if let Some(idx) = guard.selected_index {
						let vp_ctrl = guard.vp_ctrl.clone();
						if let Some(entity) = guard.scene.entities.get_mut(idx) {
							apply(entity, idx, &vp_ctrl, val);
							guard.dirty = true;
						}
					}
				}
				let state_inner = Arc::clone(&state);
				tasks.push_tree(move |tree| {
					update_header_and_status(tree, &state_inner, None);
				});
			})
			.build(),
	);
	tree.set_anchor(id, Some(Anchor::TopFill));
}


#[allow(clippy::too_many_arguments)]
pub fn insert_combo_field<F>(
	tree: &mut UiTree,
	parent: ObjectID,
	title: &str,
	items: &[&str],
	selected: usize,
	rebuild_on_change: bool,
	state: Arc<Mutex<EditorState>>,
	tasks: TaskQueue,
	apply: F,
) where
	F: Fn(&mut EntityDesc, usize, &ViewportController, String) + Send + Sync + 'static,
{
	let id = tree.insert_child(
		parent,
		ComboBox::builder()
			.title(title)
			.items(items.iter().copied())
			.selected(selected)
			.on_changed(move |_sel: usize, label: String| {
				{
					let mut guard = state.lock().unwrap();
					if let Some(idx) = guard.selected_index {
						let vp_ctrl = guard.vp_ctrl.clone();
						if let Some(entity) = guard.scene.entities.get_mut(idx) {
							apply(entity, idx, &vp_ctrl, label);
							guard.dirty = true;
						}
					}
				}
				let state_inner = Arc::clone(&state);
				let tasks_inner = tasks.clone();
				tasks.push_tree(move |tree| {
					update_header_and_status(tree, &state_inner, None);
					if rebuild_on_change {
						rebuild_inspector(tree, &state_inner, &tasks_inner);
					}
				});
			})
			.build(),
	);
	tree.set_anchor(id, Some(Anchor::TopFill));
}


/// Converts quaternion `[x, y, z, w]` to Euler angles `[pitch_x, yaw_y, roll_z]` in degrees.
pub fn quat_to_euler_deg(q: [f32; 4]) -> [f32; 3] {
	let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
	let sinr_cosp = 2.0 * (w * x + y * z);
	let cosr_cosp = 1.0 - 2.0 * (x * x + y * y);
	let pitch = sinr_cosp.atan2(cosr_cosp);

	let sinp = (2.0 * (w * y - z * x)).clamp(-1.0, 1.0);
	let yaw = sinp.asin();

	let siny_cosp = 2.0 * (w * z + x * y);
	let cosy_cosp = 1.0 - 2.0 * (y * y + z * z);
	let roll = siny_cosp.atan2(cosy_cosp);

	[pitch.to_degrees(), yaw.to_degrees(), roll.to_degrees()]
}


/// Converts Euler angles `[pitch_x, yaw_y, roll_z]` in degrees to unit quaternion `[x, y, z, w]`.
pub fn euler_deg_to_quat(euler_deg: [f32; 3]) -> [f32; 4] {
	let (hr, hp, hy) = (
		euler_deg[0].to_radians() * 0.5,
		euler_deg[1].to_radians() * 0.5,
		euler_deg[2].to_radians() * 0.5,
	);
	let (sr, cr) = hr.sin_cos();
	let (sp, cp) = hp.sin_cos();
	let (sy, cy) = hy.sin_cos();

	let w = cr * cp * cy + sr * sp * sy;
	let x = sr * cp * cy - cr * sp * sy;
	let y = cr * sp * cy + sr * cp * sy;
	let z = cr * cp * sy - sr * sp * cy;
	let len = (x * x + y * y + z * z + w * w).sqrt().max(1e-6);
	[x / len, y / len, z / len, w / len]
}
