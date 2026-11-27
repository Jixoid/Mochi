/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};

use nimbus::core::UiTree;
use nimbus::theme::{DColor, MonetRole};
use nimbus::ui::{TreeItem, TreeView};

use super::state::{EditorState, entity_badge_and_color, entity_display_name};


pub fn rebuild_hierarchy(tree: &mut UiTree, state: &Arc<Mutex<EditorState>>) {
	let (ids, scene_name, items, selected) = {
		let guard = state.lock().unwrap();
		let Some(ids) = guard.ids else { return };
		let items: Vec<TreeItem> = guard
			.scene
			.entities
			.iter()
			.enumerate()
			.map(|(i, e)| {
				let (badge, color) = entity_badge_and_color(e);
				TreeItem::new(entity_display_name(e, i))
					.with_id(i.to_string())
					.badge(badge)
					.color(color)
			})
			.collect();
		(ids, guard.scene.name.clone(), items, guard.selected_index)
	};

	if let Some(tv) = tree.get_widget_mut::<TreeView>(ids.hierarchy_tree) {
		let root = TreeItem::new(scene_name)
			.with_id("root")
			.expanded(true)
			.badge("Scene")
			.color(DColor::from(MonetRole::Primary))
			.children(items);
		tv.items = vec![root];
		tv.selected = selected.map(|i| vec![0, i]);
	}

	tree.needs_layout = true;
	tree.needs_redraw = true;
}
