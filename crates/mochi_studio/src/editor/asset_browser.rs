/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};

use nimbus::core::{Anchor, UiTree};
use nimbus::theme::{DColor, MonetRole};
use nimbus::ui::{Button, ButtonVariant, Text};

use crate::projects;

use super::state::{EditorState, clear_children};


pub fn populate_asset_browser(tree: &mut UiTree, state: &Arc<Mutex<EditorState>>) {
	let (list_id, search_root) = {
		let guard = state.lock().unwrap();
		let Some(ids) = guard.ids else { return };
		(ids.asset_browser_list, guard.entry.project_dir.clone())
	};

	clear_children(tree, list_id);
	let scenes = projects::discover_scenes(&search_root);
	if scenes.is_empty() {
		tree.insert_child(
			list_id,
			Text::builder()
				.text("Bu projede başka sahne bulunamadı.")
				.font_size(12.0)
				.color(DColor::from(MonetRole::OnSurfaceVariant))
				.build(),
		);
		return;
	}

	for entry in scenes {
		let open_path = entry.scene_path.clone();
		let btn_id = tree.insert_child(
			list_id,
			Button::builder()
				.text(format!("{}  —  {}", entry.title, entry.display_path))
				.font_size(12.0)
				.variant(ButtonVariant::Outlined)
				.on_click(move || {
					projects::launch_editor(&open_path);
				})
				.build(),
		);
		tree.set_anchor(btn_id, Some(Anchor::TopFill));
	}
	tree.needs_layout = true;
	tree.needs_redraw = true;
}
