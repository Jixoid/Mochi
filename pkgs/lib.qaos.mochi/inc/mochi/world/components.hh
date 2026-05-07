/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "mochi/geometry.hh"
#include "mochi/asset/mesh.hh"
#include "mochi/asset/texture.hh"
#include "mochi/rhi/pipeline.hh"
#include "mochi/rhi/buffer.hh"
#include <vulkan/vulkan_raii.hpp>
#include "src/entt/entt.hpp"



namespace mochi
{

  /** @brief Represents the local and global transform of an entity. */
  struct TransformComponent {
    mat4<f32> model = mat4<f32>();
  };


  /** @brief Represents the hierarchy relationship of an entity. */
  struct HierarchyComponent {
    entt::entity parent = entt::null;
    std::vector<entt::entity> children;
  };


  /** @brief Represents a renderable entity with geometry and material data. */
  struct RenderableComponent {
    sptr<asset::mesh> mesh;
    sptr<asset::texture2> texture;
    sptr<rhi::pipeline> pipeline;
    sptr<vk::raii::DescriptorSets> desc_sets;
  };


  /** @brief Represents a camera entity. */
  struct CameraComponent {
    f32 fov = 45.0f;
    f32 near = 0.1f;
    f32 far = 1000.0f;
    
    mat4<f32> view = mat4<f32>();
    mat4<f32> proj = mat4<f32>();
  };


  /** @brief Represents a light entity. */
  struct LightComponent {
    vec3<f32> color = {1.0f, 1.0f, 1.0f};
    f32 intensity = 1.0f;
  };


  // --- Raw Data Structures for UBOs ---
  
  /** @brief Represents the raw camera data passed to shaders (View and Projection matrices). */
  struct camera_t {
    mat4<f32> view;
    mat4<f32> proj;
  };
  
  /** @brief Buffer info describing the camera_t memory layout. */
  extern rhi::info<rhi::buffer> camera_i;


  /** @brief Represents the raw point light data passed to shaders. */
  struct light_t {
    vec4<f32> position;
    vec4<f32> color;
  };
  
  /** @brief Buffer info describing the light_t memory layout. */
  extern rhi::info<rhi::buffer> light_i;

}
