/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include <GLFW/glfw3.h>
#include <iostream>
#include "mochi/asset/texture.hh"
#include "mochi/core.hh"
#include "mochi/except.hh"
#include "mochi/module/window.hh"
#include "mochi/module/device.hh"
#include "mochi/module/memory.hh"
#include "mochi/world/components.hh"
#include "mochi/asset/mesh.hh"
#include "mochi/rhi/pipeline.hh"
#include "mochi/rhi/shader.hh"
#include "mochi/entity/camera_3d.hh"
#include "mochi/entity/omni_light_3d.hh"
#include "mochi/entity/mesh_instance_3d.hh"
#include "vulkan/vulkan.hpp"


using namespace mochi;



int Main()
{
  mochi::vec3<f32> cam_pos{0,0,4};
  mochi::vec3<f32> cam_front{-1};
  cam_front = cam_front.normalize();
  mochi::vec3<f32> cam_up{0,1,0};
  f32 cam_speed = 10;
  f32 yaw = -135;
  f32 pitch = -35; 
  f64 last_x = 400, last_y = 300;
  bool first_mouse = true;
  f32 mouse_sensitivity = 0.1;

  sptr<entity::Camera3D> camera;
  std::vector<sptr<entity::Node>> scene_nodes;  

  core eng(
    [](vk::raii::PhysicalDevices devices) -> vk::raii::PhysicalDevice
    {
      if (devices.empty())
        throw mochi::rhi_error("Vulkan supported graphics unit not found.");

      return devices[0];
    },
    [&](f32 dt)
    {
      if (!camera) return;

      auto win = eng.sub<module::window>().glfw();


      auto is_key_pressed = [win](int key) { return glfwGetKey(win, key) == GLFW_PRESS; };
      auto disable_cursor = [win]() { glfwSetInputMode(win, GLFW_CURSOR, GLFW_CURSOR_DISABLED); };
      auto enable_cursor = [win]() { glfwSetInputMode(win, GLFW_CURSOR, GLFW_CURSOR_NORMAL); };
      auto get_cursor_pos = [win](f64 &x, f64 &y) { glfwGetCursorPos(win, &x, &y); };


      static bool cursor_locked = false;
      static bool esc_pressed_last_frame = false;
      
      bool esc_pressed = is_key_pressed(GLFW_KEY_ESCAPE);
      
      
      if (esc_pressed && !esc_pressed_last_frame) {
        cursor_locked = !cursor_locked;
        
        if (cursor_locked) {
          disable_cursor();
          first_mouse = true;
        } else {
          enable_cursor();
        }
      }
      esc_pressed_last_frame = esc_pressed;

    
      double xpos, ypos;
      get_cursor_pos(xpos, ypos);

      if (first_mouse) {
        last_x = xpos;
        last_y = ypos;
        first_mouse = false;
      }

      float xoffset = xpos - last_x;
      float yoffset = last_y - ypos; 
      last_x = xpos;
      last_y = ypos;

      
      if (cursor_locked) {
        xoffset *= mouse_sensitivity;
        yoffset *= mouse_sensitivity;

        yaw += xoffset;
        pitch += yoffset;

        if (pitch > 89.0f) pitch = 89.0f;
        if (pitch < -89.0f) pitch = -89.0f;
        
        mochi::vec3<f32> front;
        front.X = std::cos(yaw * (M_PI / 180.0f)) * std::cos(pitch * (M_PI / 180.0f));
        front.Y = std::sin(pitch * (M_PI / 180.0f));
        front.Z = std::sin(yaw * (M_PI / 180.0f)) * std::cos(pitch * (M_PI / 180.0f));
        cam_front = front.normalize();
      }

      
      mochi::vec3<f32> cam_right = cam_front.cross(cam_up).normalize();

      bool moved = false;
      
      
      if (is_key_pressed(GLFW_KEY_W)) { cam_pos += cam_front * (cam_speed * dt); moved = true; }
      if (is_key_pressed(GLFW_KEY_S)) { cam_pos -= cam_front * (cam_speed * dt); moved = true; }
      if (is_key_pressed(GLFW_KEY_A)) { cam_pos -= cam_right * (cam_speed * dt); moved = true; }
      if (is_key_pressed(GLFW_KEY_D)) { cam_pos += cam_right * (cam_speed * dt); moved = true; }
      
      if (is_key_pressed(GLFW_KEY_SPACE)) { cam_pos += cam_up * (cam_speed * dt); moved = true; }
      if (is_key_pressed(GLFW_KEY_LEFT_SHIFT)) { cam_pos -= cam_up * (cam_speed * dt); moved = true; }

      if (moved || cursor_locked) {
        auto view_matrix = mochi::mat4<f32>::lookAt(cam_pos, cam_pos + cam_front, cam_up);
        eng.registry().get<CameraComponent>(camera->entity()).view = view_matrix;
      }
    }
  );





  auto &reg = eng.registry();

  camera = make_sptr<entity::Camera3D>(eng);
  camera->set_fov(90.0f);
  camera->set_near(0.1f);
  camera->set_far(1000.0f);
  
  auto &cam_comp = reg.get<CameraComponent>(camera->entity());
  cam_comp.view = mochi::mat4<f32>::lookAt(cam_pos, cam_pos + cam_front, cam_up);
  cam_comp.proj = mochi::mat4<f32>::perspective(camera->get_fov(), 800.0f / 600.0f, camera->get_near(), camera->get_far());


  auto make_light = [&](vec3<f32> pos) {
    auto light = make_sptr<entity::OmniLight3D>(eng);
    light->set_position(pos);
    light->set_color({1,1,1});
    light->set_intensity(10.0f);
    return light;
  };
  scene_nodes.push_back(make_light({4,0,0}));
  scene_nodes.push_back(make_light({0,4,0}));
  scene_nodes.push_back(make_light({-4,0,0}));
  scene_nodes.push_back(make_light({0,-4,0}));



  auto m3d = asset::mesh::make(eng, "/home/alforce/Masaüstü/Untitled.glb");
  auto txt = asset::texture2::make(eng, "/home/alforce/Masaüstü/Untitled.png");


  rhi::info<rhi::pipeline> pbr_i(
    {
      {mochi::vt::make<mochi::mat4<f32>>(), vk::ShaderStageFlagBits::eVertex | vk::ShaderStageFlagBits::eFragment},
    },
    {
      {&asset::vertex_i, vk::VertexInputRate::eVertex},
    },
    {
      {&mochi::camera_i, vk::DescriptorType::eUniformBuffer, vk::ShaderStageFlagBits::eVertex | vk::ShaderStageFlagBits::eFragment},
      {&mochi::light_i, vk::DescriptorType::eUniformBuffer, vk::ShaderStageFlagBits::eVertex | vk::ShaderStageFlagBits::eFragment},
      {nil, vk::DescriptorType::eCombinedImageSampler, vk::ShaderStageFlagBits::eFragment},
    }
  );


  std::vector<rhi::shaderSlot> shaders;
  shaders.push_back({vk::ShaderStageFlagBits::eVertex, rhi::shader(eng, "pkgs/lib.qaos.mochi/.qcache/pbr.vert.spv", "main")});
  shaders.push_back({vk::ShaderStageFlagBits::eFragment, rhi::shader(eng, "pkgs/lib.qaos.mochi/.qcache/pbr.frag.spv", "main")});
  
  auto pipe = rhi::pipeline::make(eng, &pbr_i, std::move(shaders));


  // Pre-allocate uniform buffers to accommodate all entities
  auto &mem = eng.sub<module::memory>();
  mem.m_camera_ubo = mem.load_UniformBuffer(&camera_i, 10, [](void*){});
  mem.m_light_ubo = mem.load_UniformBuffer(&light_i, 100, [](void*){});

  

  auto mesh_instance = make_sptr<entity::MeshInstance3D>(eng);
  mesh_instance->set_position({0,0,0});
  mesh_instance->set_mesh(m3d);
  mesh_instance->set_material(pipe, txt);
  scene_nodes.push_back(mesh_instance);

  vk::DescriptorSetAllocateInfo alloc_info(*pipe->desc_pool(), *pipe->desc_layout());
  
  auto &rend = reg.get<RenderableComponent>(mesh_instance->entity());
  rend.desc_sets = make_sptr<vk::raii::DescriptorSets>(eng.sub<module::device>().vdevice(), alloc_info);

  vk::DescriptorBufferInfo cam_buffer_info(mem.m_camera_ubo->get(), 0, camera_i.stride());
  vk::DescriptorBufferInfo lig_buffer_info(mem.m_light_ubo->get(), 0, mem.m_light_ubo->size());
  vk::DescriptorImageInfo image_info(*txt->data()->sampler(), *txt->data()->view(), vk::ImageLayout::eShaderReadOnlyOptimal);

  std::vector<vk::WriteDescriptorSet> writes;
  writes.push_back(vk::WriteDescriptorSet(
    *(*rend.desc_sets)[0], 0, 0, 1, vk::DescriptorType::eUniformBuffer, nil, &cam_buffer_info, nil
  ));
  writes.push_back(vk::WriteDescriptorSet(
    *(*rend.desc_sets)[0], 1, 0, 1, vk::DescriptorType::eUniformBuffer, nil, &lig_buffer_info, nil
  ));
  writes.push_back(vk::WriteDescriptorSet(
    *(*rend.desc_sets)[0], 2, 0, 1, vk::DescriptorType::eCombinedImageSampler, &image_info, nil, nil
  ));

  eng.sub<module::device>().vdevice().updateDescriptorSets(writes, nil);


  eng.run();

  return 0;
}




int main()
{
  try { return Main(); }
  
  catch (const vk::SystemError &e) {
    std::cerr << "Vulkan Error: " << e.what() << std::endl;
    return -1;
  }
  catch (const mochi::except &e) {
    std::cerr << "Mochi Error: " << e.what() << std::endl;
    return -1;
  }
  catch (const std::exception &e) {
    std::cerr << "Standard Error: " << e.what() << std::endl;
    return -1;
  }

  return 0;
}
