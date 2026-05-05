A minimalist Vulkan-based graphics engine and framework written in C++. QAOS focuses on a clean, layered architecture to simplify low-level graphics programming.

## Features

- **Vulkan RHI:** Wrapper for Vulkan objects (Buffers, Pipelines, Shaders) utilizing `vulkan_raii`.
- **Dynamic Rendering:** Built-in support for Vulkan dynamic rendering (no RenderPass/Framebuffer boilerplate).
- **PBR Pipeline:** Physically Based Rendering support with GGX and Schlick-Beckmann models.
- **Scene Graph:** A hierarchical node system for managing world objects.
- **Asset Management:** Wavefront (.obj) mesh loading support.
- **UBO Synchronization:** Automated management and synchronization of Uniform Buffer Objects for cameras and lights.
- **Cross-Platform Design:** Architecture separated into interfaces (.hh) and platform-specific implementations (.cc) via CMake.

## Architecture

The project is divided into three core layers:

1. **RHI (Render Hardware Interface):** Low-level GPU resource management (Buffers, Pipelines).
2. **World:** Scene objects including Nodes, Cameras, Lights, and Visuals.
3. **Asset:** High-level data structures like Meshes.

## Visuals

![Suzanne 3D Model](/images/Suzanne.png)
*PBR rendering test.*


## Building

The project uses CMake for build configuration.

```bash
mkdir build && cd build
cmake ..
make
```

## License

This project is licensed under the GNU General Public License version 3 (GPL3). See the LICENSE file for details.

Copyright (c) 2025-2026 Kadir Aydın.


### QAOS

Built with ❤️ by the open source community.
