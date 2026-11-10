/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use crate::material::template::MaterialTemplate;
use crate::material::types::{BlendMode, PbrMaterialGpuData, PbrProperties};
use mochi_math::{Vec3, Vec4};


static NEXT_INSTANCE_ID: AtomicU32 = AtomicU32::new(1);

pub struct MaterialInstance {
	id: u32,
	template: Arc<MaterialTemplate>,
	properties: RwLock<PbrProperties>,
	render_priority: AtomicI32,
}

impl MaterialInstance {
	pub fn new(template: Arc<MaterialTemplate>, properties: PbrProperties) -> Arc<Self> {
		let id = NEXT_INSTANCE_ID.fetch_add(1, Ordering::Relaxed);
		Arc::new(Self {
			id,
			template,
			properties: RwLock::new(properties),
			render_priority: AtomicI32::new(0),
		})
	}

	pub fn new_default(template: Arc<MaterialTemplate>) -> Arc<Self> {
		Self::new(template, PbrProperties::default())
	}

	pub fn id(&self) -> u32 {
		self.id
	}

	pub fn template(&self) -> &Arc<MaterialTemplate> {
		&self.template
	}

	pub fn blend_mode(&self) -> BlendMode {
		self.template.blend_mode()
	}

	pub fn render_priority(&self) -> i32 {
		self.render_priority.load(Ordering::Relaxed)
	}

	pub fn set_render_priority(&self, priority: i32) {
		self.render_priority.store(priority, Ordering::Relaxed);
	}

	pub fn properties(&self) -> PbrProperties {
		self.properties.read().unwrap().clone()
	}

	pub fn set_base_color(&self, color: Vec4<f32>) {
		self.properties.write().unwrap().base_color = color;
	}

	pub fn set_roughness(&self, roughness: f32) {
		self.properties.write().unwrap().roughness = roughness;
	}

	pub fn set_metallic(&self, metallic: f32) {
		self.properties.write().unwrap().metallic = metallic;
	}

	pub fn set_emissive(&self, color: Vec3<f32>, intensity: f32) {
		let mut props = self.properties.write().unwrap();
		props.emissive = color;
		props.emissive_intensity = intensity;
	}

	pub fn to_gpu_data(&self) -> PbrMaterialGpuData {
		self.properties.read().unwrap().to_gpu_data()
	}
}
