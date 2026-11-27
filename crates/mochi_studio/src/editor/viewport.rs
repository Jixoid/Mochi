/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};

use nimbus::core::UiTree;
use nimbus::ui::Text;
use nimbus::ext::gizmo::{
	GizmoCamera, OrientationGizmo, Quat as GizmoQuat, Rad as GizmoRad, Transform as GizmoTransform,
	TransformGizmo, Vec3 as GizmoVec3,
};

use super::state::EditorState;


pub fn update_header_and_status(
	tree: &mut UiTree,
	state: &Arc<Mutex<EditorState>>,
	status_override: Option<String>,
) {
	let (ids, title_text) = {
		let guard = state.lock().unwrap();
		let Some(ids) = guard.ids else { return };
		let star = if guard.dirty { " *" } else { "" };
		(ids, format!("{}{}", guard.scene.name, star))
	};

	if let Some(title) = tree.get_widget_mut::<Text>(ids.scene_title) {
		title.text = title_text;
	}
	if let Some(msg) = status_override
		&& let Some(status) = tree.get_widget_mut::<Text>(ids.status_text)
	{
		status.text = msg;
	}

	tree.needs_layout = true;
	tree.needs_redraw = true;
}


pub fn sync_gizmo_camera(tree: &mut UiTree, state: &Arc<Mutex<EditorState>>, animate: bool) {
	let (ids, w, h, yaw_deg, pitch_deg, dist, target) = {
		let guard = state.lock().unwrap();
		let Some(ids) = guard.ids else { return };
		(
			ids,
			guard.viewport_size.0,
			guard.viewport_size.1,
			guard.camera.yaw_deg,
			guard.camera.pitch_deg,
			guard.camera.distance,
			guard.camera.target,
		)
	};

	let aspect = if h > 0 {
		w as f32 / h as f32
	} else {
		16.0 / 9.0
	};
	let cam = GizmoCamera::from_orbit(
		GizmoVec3::new(target[0], target[1], target[2]),
		dist,
		yaw_deg.to_radians(),
		pitch_deg.to_radians(),
		GizmoRad::new(60.0f32.to_radians()),
		aspect,
	);

	if let Some(tg) = tree.get_widget_mut::<TransformGizmo>(ids.transform_gizmo) {
		if animate {
			tg.animate_camera(cam);
		} else {
			tg.set_camera(cam);
		}
	}
	if let Some(og) = tree.get_widget_mut::<OrientationGizmo>(ids.orientation_gizmo) {
		if animate {
			og.animate_camera(cam);
		} else {
			og.set_camera(cam);
		}
	}
	tree.needs_redraw = true;
}


pub fn sync_gizmo_from_selection(tree: &mut UiTree, state: &Arc<Mutex<EditorState>>) {
	let (ids, tf_opt) = {
		let guard = state.lock().unwrap();
		let Some(ids) = guard.ids else { return };
		let tf_opt = guard
			.selected_index
			.and_then(|idx| guard.scene.entities.get(idx))
			.map(|e| e.transform.clone().unwrap_or_default());
		(ids, tf_opt)
	};

	match tf_opt {
		Some(tf) => {
			tree.set_visible(ids.transform_gizmo, true);
			if let Some(tg) = tree.get_widget_mut::<TransformGizmo>(ids.transform_gizmo) {
				tg.transform = GizmoTransform {
					translation: GizmoVec3::new(tf.translation[0], tf.translation[1], tf.translation[2]),
					rotation: GizmoQuat::new(
						tf.rotation[0],
						tf.rotation[1],
						tf.rotation[2],
						tf.rotation[3],
					),
					scale: GizmoVec3::new(tf.scale[0], tf.scale[1], tf.scale[2]),
				};
			}
		}
		None => {
			tree.set_visible(ids.transform_gizmo, false);
		}
	}
	tree.needs_redraw = true;
}


pub fn update_viewport_hud(tree: &mut UiTree, state: &Arc<Mutex<EditorState>>) {
	let (ids, w, h, count, yaw, pitch, dist) = {
		let guard = state.lock().unwrap();
		let Some(ids) = guard.ids else { return };
		(
			ids,
			guard.viewport_size.0,
			guard.viewport_size.1,
			guard.scene.entities.len(),
			guard.camera.yaw_deg,
			guard.camera.pitch_deg,
			guard.camera.distance,
		)
	};

	if let Some(hud) = tree.get_widget_mut::<Text>(ids.viewport_hud) {
		hud.text = format!("Viewport • {w}x{h} • {count} nesne");
	}
	if let Some(cam_hud) = tree.get_widget_mut::<Text>(ids.viewport_camera_hud) {
		cam_hud.text = format!("Orbit: Yaw {yaw:.1}°  Pitch {pitch:.1}°  Zoom {dist:.1}");
	}
	if let Some(size_txt) = tree.get_widget_mut::<Text>(ids.viewport_size_text) {
		size_txt.text = format!("Viewport: {w}x{h}");
	}

	tree.needs_layout = true;
	tree.needs_redraw = true;
}


pub fn focus_selected_entity(tree: &mut UiTree, state: &Arc<Mutex<EditorState>>) {
	let focused = {
		let mut guard = state.lock().unwrap();
		let target_pos = guard
			.selected_index
			.and_then(|i| guard.scene.entities.get(i))
			.and_then(|e| e.transform.as_ref())
			.map(|tf| tf.translation)
			.unwrap_or([0.0, 0.0, 0.0]);
		guard.camera.target = target_pos;
		guard.camera.distance = guard.camera.distance.clamp(3.5, 8.0);
		let (yaw, pitch, dist, target) = (
			guard.camera.yaw_deg,
			guard.camera.pitch_deg,
			guard.camera.distance,
			guard.camera.target,
		);
		guard.vp_ctrl.update_camera(yaw, pitch, dist, target);
		target
	};
	sync_gizmo_camera(tree, state, true);
	update_viewport_hud(tree, state);
	update_header_and_status(
		tree,
		state,
		Some(format!(
			"Odaklanıldı: ({:.1}, {:.1}, {:.1})",
			focused[0], focused[1], focused[2]
		)),
	);
}
