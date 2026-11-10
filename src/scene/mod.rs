/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod components;
pub mod mesh;

pub use components::*;
pub use mesh::*;
pub use hecs::Entity;


pub struct Scene {
	world: hecs::World,
}

impl Scene {
	pub fn new() -> Self {
		Self {
			world: hecs::World::new(),
		}
	}

	pub fn world(&self) -> &hecs::World {
		&self.world
	}

	pub fn world_mut(&mut self) -> &mut hecs::World {
		&mut self.world
	}

	pub fn spawn(&mut self, components: impl hecs::DynamicBundle) -> Entity {
		self.world.spawn(components)
	}

	pub fn despawn(&mut self, entity: Entity) -> Result<(), hecs::NoSuchEntity> {
		self.world.despawn(entity)
	}

	pub fn active_camera(&self) -> Option<(Entity, TransformComponent, CameraComponent)> {
		for (entity, (transform, camera)) in self.world.query::<(&TransformComponent, &CameraComponent)>().iter() {
			if camera.is_active {
				return Some((entity, *transform, *camera));
			}
		}
		None
	}
}

impl Default for Scene {
	fn default() -> Self {
		Self::new()
	}
}
