/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/except.hh"
#include "mochi/module/bridge.hh"
#include "mochi/module/device.hh"
#include "mochi/module/memory.hh"
#include "mochi/module/display.hh"
#include <GLFW/glfw3.h>
#include <string_view>



namespace mochi::module
{

  fun __attribute__((constructor())) glfw_init() { glfwInit(); }
  fun __attribute__((destructor()))  glfw_fini() { glfwTerminate(); }



  display::display(module::bridge &bridge, module::device &device, module::memory &memory, std::string_view title, int width, int height)
    : m_device(device)
    , m_memory(memory)
  {
    glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);

    m_window = glfwCreateWindow(
      width, height,
      std::string(title).c_str(),
      nil, nil
    );

    glfwGetFramebufferSize(m_window, &m_width, &m_height);

    glfwSetWindowUserPointer(m_window, this);

    glfwSetFramebufferSizeCallback(m_window, framebuffer_resize_callback);

    VkSurfaceKHR rawSurface;
    if (glfwCreateWindowSurface(*bridge.inst(), m_window, nil, &rawSurface) != VK_SUCCESS)
      throw mochi::rhi_error("Failed to create surface!");
    
    vk_surface = vk::raii::SurfaceKHR(bridge.inst(), rawSurface);

    recreate_swapchain();
  }

  display::~display()
  {
    if (m_depth_image) {
      vmaDestroyImage(m_memory.allocator(), m_depth_image, m_depth_allocation);
      m_depth_image = nil;
      m_depth_allocation = nil;
    }
  }



  fun display::recreate_swapchain() -> void
  {
    // Wait if window is minimized
    int width = 0, height = 0;
    glfwGetFramebufferSize(m_window, &width, &height);
    while (width == 0 || height == 0) {
      glfwGetFramebufferSize(m_window, &width, &height);
      glfwWaitEvents();
    }

    m_device.vdevice().waitIdle();


    m_image_views.clear();
    m_render_finished_sems.clear();
    m_depth_view.clear();

    if (m_depth_image) {
      vmaDestroyImage(m_memory.allocator(), m_depth_image, m_depth_allocation);
      m_depth_image = nil;
      m_depth_allocation = nil;
    }


    // Smooth transition
    vk::SwapchainKHR old_swapchain = *m_swapchain;


    auto caps = m_device.phys_dev().getSurfaceCapabilitiesKHR(vk_surface);
    auto formats = m_device.phys_dev().getSurfaceFormatsKHR(vk_surface);


    vk::SurfaceFormatKHR selected_format = formats[0];
    for (const auto &f: formats)
      if (f.format == vk::Format::eR8G8B8A8Unorm && f.colorSpace == vk::ColorSpaceKHR::eSrgbNonlinear) {
        selected_format = f;
        break;
      }

    m_format = selected_format.format;



    if (caps.currentExtent.width != std::numeric_limits<u32>::max())
      m_extent = caps.currentExtent;
    else {
      m_extent.width = std::clamp(static_cast<u32>(width), caps.minImageExtent.width, caps.maxImageExtent.width);
      m_extent.height = std::clamp(static_cast<u32>(height), caps.minImageExtent.height, caps.maxImageExtent.height);
    }


    m_image_count = std::max<u32>(caps.minImageCount, 3);
    if (caps.maxImageCount > 0 && m_image_count > caps.maxImageCount) {
      m_image_count = caps.maxImageCount;
    }

    vk::SwapchainCreateInfoKHR swp_info(
      {}, vk_surface, m_image_count, m_format,
      selected_format.colorSpace, m_extent, 1,
      vk::ImageUsageFlagBits::eColorAttachment
    );
    
    swp_info.setPresentMode(vk::PresentModeKHR::eFifo);
    swp_info.setPreTransform(caps.currentTransform);
    swp_info.setCompositeAlpha(vk::CompositeAlphaFlagBitsKHR::eOpaque);
    swp_info.setOldSwapchain(old_swapchain);

    m_swapchain = vk::raii::SwapchainKHR(m_device.vdevice(), swp_info);


    m_images = m_swapchain.getImages();
    vk::SemaphoreCreateInfo sem_info{};
    for (const auto &img: m_images) {
      vk::ImageViewCreateInfo view_info(
        {}, img, vk::ImageViewType::e2D, m_format,
        {}, {vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1}
      );
      m_image_views.push_back(vk::raii::ImageView(m_device.vdevice(), view_info));
      m_render_finished_sems.push_back(vk::raii::Semaphore(m_device.vdevice(), sem_info));
    }


    vk::ImageCreateInfo depth_info(
      {}, vk::ImageType::e2D, m_depth_format,
      vk::Extent3D(m_extent, 1), 1, 1,
      vk::SampleCountFlagBits::e1, vk::ImageTiling::eOptimal,
      vk::ImageUsageFlagBits::eDepthStencilAttachment
    );

    VmaAllocationCreateInfo alloc_info = {};
    alloc_info.usage = VMA_MEMORY_USAGE_AUTO_PREFER_DEVICE;
    alloc_info.requiredFlags = VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT;

    // 3. VMA üzerinden tek hamlede hem Image hem Memory oluştur ve bağla
    VkImageCreateInfo raw_depth_info = static_cast<VkImageCreateInfo>(depth_info);
    VkImage raw_image;
    
    vmaCreateImage(m_memory.allocator(), &raw_depth_info, &alloc_info, &raw_image, &m_depth_allocation, nullptr);
    m_depth_image = raw_image;

    // 4. ImageView'u oluştur (RAII ile devam edebilirsin)
    vk::ImageViewCreateInfo depth_view_info(
      {}, m_depth_image, vk::ImageViewType::e2D, m_depth_format,
      {}, {vk::ImageAspectFlagBits::eDepth, 0, 1, 0, 1}
    );
    m_depth_view = vk::raii::ImageView(m_device.vdevice(), depth_view_info);

    m_resized = false;
  }


  void display::framebuffer_resize_callback(GLFWwindow* win, int width, int height)
  {
    auto app = reinterpret_cast<display*>(glfwGetWindowUserPointer(win));
    
    app->m_resized = true;
    app->m_width = width;
    app->m_height = height;
  }

  fun display::proc_events() -> bool
  {
    glfwPollEvents();

    if (m_width == 0 || m_height == 0) {
      glfwWaitEvents();
      return !glfwWindowShouldClose(m_window);
    }

    return !glfwWindowShouldClose(m_window);
  }

  fun display::acquire_next_image(vk::Semaphore image_available_sem) -> u32
  {
    try {
      auto [result, img_idx] = m_swapchain.acquireNextImage(
        UINT64_MAX, 
        image_available_sem, 
        nil
      );
      if (result == vk::Result::eErrorOutOfDateKHR || result == vk::Result::eSuboptimalKHR) {
        recreate_swapchain();
        return acquire_next_image(image_available_sem);
      }
      return img_idx;
    } catch (const vk::OutOfDateKHRError &e) {
      recreate_swapchain();
      return acquire_next_image(image_available_sem);
    }
  }

  fun display::present(vk::Semaphore render_finished_sem, u32 image_index) -> void
  {
    vk::PresentInfoKHR present_info(
      1, &render_finished_sem,
      1, &*m_swapchain,
      &image_index
    );

    try {
      auto err = m_device.graphics_q().best().queue.presentKHR(present_info);

      if (err == vk::Result::eErrorOutOfDateKHR || err == vk::Result::eSuboptimalKHR || m_resized)
        recreate_swapchain();
      
      else if (err != vk::Result::eSuccess)
        throw mochi::rhi_error("Critical error while presenting image to screen!");
    }
    catch (const vk::OutOfDateKHRError &e) {
      recreate_swapchain();
    }
    catch (const vk::SystemError &e) {
      if (e.code() == vk::Result::eErrorOutOfDateKHR || e.code() == vk::Result::eSuboptimalKHR)
        recreate_swapchain();
      else
        throw e;
    }
  }

  fun display::get_render_target(u32 image_index) -> rhi::render_target
  {
    return rhi::render_target{
      m_images[image_index],
      *m_image_views[image_index],
      m_format,
      m_depth_image,
      *m_depth_view,
      m_depth_format,
      m_extent,
      vk::ImageLayout::ePresentSrcKHR
    };
  }

  fun display::get_render_finished_sem(u32 image_index) -> vk::Semaphore
  {
    return *m_render_finished_sems[image_index];
  }
  
}
