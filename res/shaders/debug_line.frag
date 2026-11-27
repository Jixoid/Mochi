/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


#version 450

layout(location = 0) in vec4 in_color;
layout(location = 0) out vec4 out_color;

void main() {
	out_color = in_color;
}
