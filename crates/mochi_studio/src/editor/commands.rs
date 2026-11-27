/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};

use nimbus::core::TaskQueue;
use nimbus::ui::{Toast, ToastKind};

use super::console::push_console_log;
use super::hierarchy::rebuild_hierarchy;
use super::inspector::rebuild_inspector;
use super::state::{EditorState, EntityPreset, HistorySnapshot, entity_display_name};
use super::viewport::{sync_gizmo_from_selection, update_header_and_status, update_viewport_hud};


pub fn save_scene(state: &Arc<Mutex<EditorState>>, tasks: &TaskQueue) {
	let res = {
		let mut guard = state.lock().unwrap();
		let path_str = guard.entry.scene_path.to_string_lossy().to_string();
		match guard.scene.save_ron(&path_str) {
			Ok(()) => {
				guard.dirty = false;
				Ok(format!("Kaydedildi: {}", guard.entry.display_path))
			}
			Err(e) => Err(format!("Kaydetme hatası: {e}")),
		}
	};
	let state = Arc::clone(state);
	tasks.push_tree(move |tree| {
		let (title, msg, kind, level) = match res {
			Ok(m) => ("Sahne Kaydedildi", m, ToastKind::Success, "OK"),
			Err(m) => ("Kaydetme Hatası", m, ToastKind::Error, "ERR"),
		};
		Toast::builder()
			.title(title)
			.message(msg.clone())
			.kind(kind)
			.show(tree);
		push_console_log(tree, &state, level, &msg);
		update_header_and_status(tree, &state, Some(msg));
	});
}


pub fn add_entity_preset(state: &Arc<Mutex<EditorState>>, tasks: &TaskQueue, preset: EntityPreset) {
	let new_name = {
		let mut guard = state.lock().unwrap();
		guard.record_undo("Nesne Ekle");
		let idx = guard.scene.entities.len() + 1;
		let entity = preset.build_entity(idx);
		let name = entity_display_name(&entity, idx - 1);
		let is_empty_only = matches!(preset, EntityPreset::Empty);
		let tf = entity.transform.clone().unwrap_or_default();
		guard.scene.entities.push(entity);
		guard.selected_index = Some(guard.scene.entities.len() - 1);
		guard.dirty = true;
		guard.vp_ctrl.set_selection(guard.selected_index);
		if is_empty_only {
			guard.vp_ctrl.add_entity(tf);
		} else {
			guard.reload_viewport_scene();
		}
		name
	};

	let state = Arc::clone(state);
	let tasks_inner = tasks.clone();
	tasks.push_tree(move |tree| {
		let msg = format!("Yeni nesne eklendi: {new_name}");
		push_console_log(tree, &state, "ECS", &msg);
		update_header_and_status(tree, &state, Some(msg));
		rebuild_hierarchy(tree, &state);
		rebuild_inspector(tree, &state, &tasks_inner);
		sync_gizmo_from_selection(tree, &state);
		update_viewport_hud(tree, &state);
	});
}


pub fn duplicate_selected_entity(state: &Arc<Mutex<EditorState>>, tasks: &TaskQueue) {
	let dup_name = {
		let mut guard = state.lock().unwrap();
		let Some(idx) = guard.selected_index else {
			return;
		};
		let Some(mut cloned) = guard.scene.entities.get(idx).cloned() else {
			return;
		};
		guard.record_undo("Nesne Kopyala");
		let base = entity_display_name(&cloned, idx);
		let new_name = format!("{base}_Copy");
		cloned.name = Some(new_name.clone());
		if let Some(tf) = &mut cloned.transform {
			tf.translation[0] += 1.0;
		}
		guard.scene.entities.push(cloned);
		guard.selected_index = Some(guard.scene.entities.len() - 1);
		guard.dirty = true;
		guard.vp_ctrl.set_selection(guard.selected_index);
		guard.reload_viewport_scene();
		new_name
	};

	let state = Arc::clone(state);
	let tasks_inner = tasks.clone();
	tasks.push_tree(move |tree| {
		let msg = format!("Nesne kopyalandı: {dup_name}");
		push_console_log(tree, &state, "ECS", &msg);
		update_header_and_status(tree, &state, Some(msg));
		rebuild_hierarchy(tree, &state);
		rebuild_inspector(tree, &state, &tasks_inner);
		sync_gizmo_from_selection(tree, &state);
		update_viewport_hud(tree, &state);
	});
}


pub fn delete_selected_entity(state: &Arc<Mutex<EditorState>>, tasks: &TaskQueue) {
	let removed_name = {
		let mut guard = state.lock().unwrap();
		if let Some(idx) = guard.selected_index
			&& idx < guard.scene.entities.len()
		{
			guard.record_undo("Nesne Sil");
			let removed = guard.scene.entities.remove(idx);
			guard.dirty = true;
			guard.vp_ctrl.remove_entity(idx);
			guard.selected_index = if guard.scene.entities.is_empty() {
				None
			} else {
				Some(idx.min(guard.scene.entities.len() - 1))
			};
			guard.vp_ctrl.set_selection(guard.selected_index);
			Some(entity_display_name(&removed, idx))
		} else {
			None
		}
	};

	if let Some(name) = removed_name {
		let state = Arc::clone(state);
		let tasks_inner = tasks.clone();
		tasks.push_tree(move |tree| {
			let msg = format!("Nesne silindi: {name}");
			push_console_log(tree, &state, "ECS", &msg);
			update_header_and_status(tree, &state, Some(msg));
			rebuild_hierarchy(tree, &state);
			rebuild_inspector(tree, &state, &tasks_inner);
			sync_gizmo_from_selection(tree, &state);
			update_viewport_hud(tree, &state);
		});
	}
}


pub fn undo(state: &Arc<Mutex<EditorState>>, tasks: &TaskQueue) {
	let label = {
		let mut guard = state.lock().unwrap();
		let Some(prev) = guard.history.undo.pop() else {
			return;
		};
		let current = HistorySnapshot {
			scene: guard.scene.clone(),
			selected_index: guard.selected_index,
			label: prev.label.clone(),
		};
		guard.history.redo.push(current);
		guard.scene = prev.scene;
		guard.selected_index = prev.selected_index;
		guard.dirty = true;
		guard.vp_ctrl.set_selection(guard.selected_index);
		guard.reload_viewport_scene();
		prev.label
	};

	let state = Arc::clone(state);
	let tasks_inner = tasks.clone();
	tasks.push_tree(move |tree| {
		let msg = format!("Geri alındı: {label}");
		push_console_log(tree, &state, "UNDO", &msg);
		update_header_and_status(tree, &state, Some(msg));
		rebuild_hierarchy(tree, &state);
		rebuild_inspector(tree, &state, &tasks_inner);
		sync_gizmo_from_selection(tree, &state);
		update_viewport_hud(tree, &state);
	});
}


pub fn redo(state: &Arc<Mutex<EditorState>>, tasks: &TaskQueue) {
	let label = {
		let mut guard = state.lock().unwrap();
		let Some(next) = guard.history.redo.pop() else {
			return;
		};
		let current = HistorySnapshot {
			scene: guard.scene.clone(),
			selected_index: guard.selected_index,
			label: next.label.clone(),
		};
		guard.history.undo.push(current);
		guard.scene = next.scene;
		guard.selected_index = next.selected_index;
		guard.dirty = true;
		guard.vp_ctrl.set_selection(guard.selected_index);
		guard.reload_viewport_scene();
		next.label
	};

	let state = Arc::clone(state);
	let tasks_inner = tasks.clone();
	tasks.push_tree(move |tree| {
		let msg = format!("Yinelendi: {label}");
		push_console_log(tree, &state, "UNDO", &msg);
		update_header_and_status(tree, &state, Some(msg));
		rebuild_hierarchy(tree, &state);
		rebuild_inspector(tree, &state, &tasks_inner);
		sync_gizmo_from_selection(tree, &state);
		update_viewport_hud(tree, &state);
	});
}
