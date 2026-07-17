/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include <GLFW/glfw3.h>
#include <iostream>
#include <random>
#include "mochi/core/core.hh"
#include "mochi/ecs/multi_mesh.hh"
#include "mochi/except.hh"
#include "mochi/asset/mesh.hh"
#include "mochi/ecs/camera.hh"
#include "mochi/ecs/point_light.hh"
#include "mochi/ecs/transform.hh"
#include "mochi/manager/window_manager.hh"
#include "mochi/math/quaternion.hh"
#include "mochi/math/vec3.hh"
#include "mochi/vfs/vfs.hh"
#include "mochi/rhi/manager/alloc_manager.hh"
#include "mochi/rhi/manager/transfer_manager.hh"



fun Main() -> int {
  mochi::Core Mochi;
  auto &Reg = Mochi.registry();


  mochi::vec3<f32> cam_pos{0,0,4};
  mochi::vec3<f32> cam_front{-1};
  cam_front = cam_front.normalize();
  mochi::vec3<f32> cam_up{0,1,0};
  f32 cam_speed = 2;
  f32 yaw = -135;
  f32 pitch = -35; 
  f64 last_x = 400, last_y = 300;
  bool first_mouse = true;
  f32 mouse_sensitivity = 0.1;

  entt::entity camera = entt::null;
  std::vector<entt::entity> scene_nodes;

  camera = Reg.create();
  auto &cam_comp = Reg.emplace<mochi::ecs::Camera>(camera);
  Reg.emplace<mochi::ecs::Transform>(camera);
  cam_comp.fov = 90.0f;
  cam_comp.near = 0.1f;
  cam_comp.far = 1000.0f;
  cam_comp.view = mochi::mat4x4<f32>::lookAt(cam_pos, cam_pos + cam_front, cam_up);
  cam_comp.proj = mochi::mat4x4<f32>::perspective(cam_comp.fov, 800.0f / 600.0f, cam_comp.near, cam_comp.far);


  auto make_light = [&](mochi::vec3<f32> pos) {
    auto light = Reg.create();
    auto &light_comp = Reg.emplace<mochi::ecs::PointLight>(light);
    light_comp.color = {1,1,1};
    light_comp.intensity = 10.0f;
    auto &transform = Reg.emplace<mochi::ecs::Transform>(light);
    transform.model = mochi::mat4x3<f32>::model(pos, mochi::quaternion<f32>(), {1});
    return light;
  };
  scene_nodes.push_back(make_light({4,0,0}));
  scene_nodes.push_back(make_light({0,4,0}));
  scene_nodes.push_back(make_light({-4,0,0}));
  scene_nodes.push_back(make_light({0,-4,0}));


  auto m3d = mochi::asset::Mesh::make(Mochi, "file:///home/alforce/Masaüstü/Untitled.glb"_vfs_map->span(), ".glb");
  for (auto& mat : m3d->material()) mat->setCount(mochi::rhi::MaterialCount::Multi);


  const u32 PARTICLE_COUNT = 5;
  std::vector<mochi::ecs::instance_data_t> instance_data(PARTICLE_COUNT);
  
  std::mt19937 rnd(1337);
  std::uniform_real_distribution<f32> dist(-5.0f, 5.0f);
  
  for (u32 i = 0; i < PARTICLE_COUNT; i++)
    instance_data[i].pos_radius = {dist(rnd), dist(rnd) + 10.0f, dist(rnd), 0.1f};
  

  auto instance_buffer = Mochi.sub<mochi::rhi::AllocManager>().allocBuffer(
    sizeof(mochi::ecs::instance_data_t) * PARTICLE_COUNT,
    mochi::rhi::BufferUsage::DeviceAddress | mochi::rhi::BufferUsage::TransferDst,
    mochi::rhi::AllocationCreate::Mapped | mochi::rhi::AllocationCreate::HostSequentialWrite,
    mochi::rhi::AllocationLocation::PreferDevice
  );

  Mochi.sub<mochi::rhi::TransferManager>().copyMemoryToBuffer(
    mochi::rhi::TransferTime::Now,
    instance_data.data(),
    instance_buffer.get()
  );




  auto mesh_instance = Reg.create();
  auto &mesh_comp = Reg.emplace<mochi::ecs::MultiMesh>(mesh_instance);
  mesh_comp.mesh = m3d;
  mesh_comp.instances = instance_buffer;
  mesh_comp.active_count = PARTICLE_COUNT;
  auto &transform = Reg.emplace<mochi::ecs::Transform>(mesh_instance);
  transform.model = mochi::mat4x3<f32>::model({0}, mochi::quaternion<f32>(), {1});
  scene_nodes.push_back(mesh_instance);


  Mochi.idle() = [&](f32 dt){
    if (camera == entt::null) return;

    auto win = Mochi.sub<mochi::manager::WindowManager>().glfw();


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
      front.x = std::cos(yaw * (M_PI / 180.0f)) * std::cos(pitch * (M_PI / 180.0f));
      front.y = std::sin(pitch * (M_PI / 180.0f));
      front.z = std::sin(yaw * (M_PI / 180.0f)) * std::cos(pitch * (M_PI / 180.0f));
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
      Mochi.registry().get<mochi::ecs::Camera>(camera).view = mochi::mat4x4<f32>::lookAt(cam_pos, cam_pos + cam_front, cam_up);
      Mochi.registry().get<mochi::ecs::Camera>(camera).proj = mochi::mat4x4<f32>::perspective(90, (f32)Mochi.sub<mochi::manager::WindowManager>().extent().x()/Mochi.sub<mochi::manager::WindowManager>().extent().y(), 0.1, 100);
    }
  };

  Mochi.run();

  scene_nodes.clear();
  m3d.reset();

  return 0;
}



fun main() -> i32 {
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
