/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


#version 450

layout(location = 0) out vec2 out_ndc;

void main() {
	const vec2 positions[3] = vec2[3](
		vec2(-1.0, -1.0),
		vec2(-1.0,  3.0),
		vec2( 3.0, -1.0)
	);
	vec2 pos = positions[gl_VertexIndex];
	out_ndc = pos;
	gl_Position = vec4(pos, 0.0, 1.0);
}
