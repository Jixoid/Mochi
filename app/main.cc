/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include <GLFW/glfw3.h>
#include <iostream>
#include "mochi/core.hh"
#include "mochi/except.hh"
#include "mochi/module/display.hh"
#include "mochi/asset/mesh.hh"
#include "mochi/ecs/camera.hh"
#include "mochi/ecs/point_light.hh"
#include "mochi/ecs/mesh.hh"
#include "mochi/ecs/transform.hh"
#include "mochi/vfs/vfs_file.hh"
#include "mochi/vfs/vfs_embed.hh"

using namespace mochi;



int Main()
{
  auto __vfs_file  = vfs::__file::get();
  auto __vfs_embed = vfs::__embed::get();


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

  core eng(
    [](vk::raii::PhysicalDevices devices) -> vk::raii::PhysicalDevice
    {
      if (devices.empty())
        throw mochi::rhi_error("Vulkan supported graphics unit not found.");

      return devices[0];
    },
    [&](f32 dt)
    {
      if (camera == entt::null) return;

      auto win = eng.sub<module::display>().glfw();


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
        eng.registry().get<ecs::Camera>(camera).view = mochi::mat4<f32>::lookAt(cam_pos, cam_pos + cam_front, cam_up);
        eng.registry().get<ecs::Camera>(camera).proj = mochi::mat4<f32>::perspective(90, (f32)eng.sub<module::display>().extent().width/eng.sub<module::display>().extent().height, 0.1, 100);
      }
    }
  );



  auto &reg = eng.registry();

  camera = reg.create();
  auto &cam_comp = reg.emplace<ecs::Camera>(camera);
  reg.emplace<ecs::Transform>(camera);
  cam_comp.fov = 90.0f;
  cam_comp.near = 0.1f;
  cam_comp.far = 1000.0f;
  cam_comp.view = mochi::mat4<f32>::lookAt(cam_pos, cam_pos + cam_front, cam_up);
  cam_comp.proj = mochi::mat4<f32>::perspective(cam_comp.fov, 800.0f / 600.0f, cam_comp.near, cam_comp.far);


  auto make_light = [&](vec3<f32> pos) {
    auto light = reg.create();
    auto &light_comp = reg.emplace<ecs::PointLight>(light);
    light_comp.color = {1,1,1};
    light_comp.intensity = 10.0f;
    auto &transform = reg.emplace<ecs::Transform>(light);
    transform.model = mochi::mat4<f32>::model(pos, quaternion<f32>(), {1});
    return light;
  };
  scene_nodes.push_back(make_light({4,0,0}));
  scene_nodes.push_back(make_light({0,4,0}));
  scene_nodes.push_back(make_light({-4,0,0}));
  scene_nodes.push_back(make_light({0,-4,0}));


  auto m3d = asset::mesh::make(eng, "file:///home/alforce/Masaüstü/Untitled.glb"_vfs_map->span(), ".glb");


  auto mesh_instance = reg.create();
  auto &mesh_comp = reg.emplace<ecs::Mesh>(mesh_instance);
  mesh_comp.mesh = m3d;
  auto &transform = reg.emplace<ecs::Transform>(mesh_instance);
  transform.model = mochi::mat4<f32>::model({0}, quaternion<f32>(), {1});
  scene_nodes.push_back(mesh_instance);


  eng.run();

  scene_nodes.clear();
  m3d.reset();

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
