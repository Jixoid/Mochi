/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};

use nimbus::core::{Anchor, UiTree};
use nimbus::math::Color;
use nimbus::theme::{DColor, MonetRole};
use nimbus::ui::Text;

use super::state::EditorState;


pub fn push_console_log(
	tree: &mut UiTree,
	state: &Arc<Mutex<EditorState>>,
	level: &str,
	msg: &str,
) {
	let Some(console_id) = state.lock().unwrap().ids.map(|i| i.console_log_list) else {
		return;
	};
	let color = match level {
		"OK" => DColor::from(Color::new(0.32, 0.78, 0.48, 1.0)),
		"ERR" => DColor::from(MonetRole::Error),
		"ECS" | "UI" | "UNDO" => DColor::from(MonetRole::Primary),
		_ => DColor::from(MonetRole::OnSurfaceVariant),
	};
	let txt_id = tree.insert_child(
		console_id,
		Text::builder()
			.text(format!("[{level}] {msg}"))
			.font_size(12.0)
			.color(color)
			.build(),
	);
	tree.set_anchor(txt_id, Some(Anchor::TopFill));
	tree.needs_layout = true;
	tree.needs_redraw = true;
}
