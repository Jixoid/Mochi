## Mochi

A modern, Vulkan 1.4-based rendering engine and framework. 
Developed under the QAOS umbrella, Mochi focuses on a clean, layered architecture to simplify low-level graphics programming while embracing modern principles.



A minimalist Vulkan-based graphics engine and framework written in C++.
QAOS focuses on a clean, layered architecture to simplify low-level graphics programming.

Other languages: [tr](README.tr.md)


## Features

Mochi has evolved beyond a minimalist wrapper, providing a robust foundation for high-performance rendering:

* **Modern RHI:** Built on Vulkan 1.4 using `vulkan-hpp` (RAII). Features Dynamic Rendering (no legacy RenderPass/Framebuffer boilerplates) and Buffer Device Address (BDA) for modern data binding.
* **Entity Component System:** Powered by `EnTT` for cache-friendly, flexible scene and object management (Transforms, Cameras, Lights, Meshes).
* **Smart Memory & Resource Management:** Integrated with Vulkan Memory Allocator (VMA). Features an automated, batched transfer queue system for zero-bottleneck asset loading.
* **Virtual File System (VFS):** A custom VFS that seamlessly resolves assets from both physical disk storage and compiled-in embedded binaries.
* **SIMD-Accelerated Math:** A custom mathematics library optimized with SSE/AVX intrinsics for heavy computational workloads.
* **Asset Pipeline:** Built-in `.gltf` / `.glb` and `.obj` parsing, alongside a ready-to-use PBR material workflow.


## Architecture

The project is structured into highly decoupled modules:

1. **RHI (Render Hardware Interface):** Low-level GPU resource management, pipelines, and descriptors.
2. **ECS (World):** Data-oriented scene entities, transforms, and lighting representation.
3. **Asset & VFS:** High-level data structures (Meshes, Textures) and hardware-agnostic file loading.
4. **Core Modules:** Unified management of the Display, Memory, Device, and Renderer loops.


## Visuals

![WaterBottle 3D Model](/images/WaterBottle.png)
*PBR rendering test.*

![Avacado 3D Model](/images/Avacado.png)
*PBR rendering test.*

![Suzanne 3D Model](/images/Suzanne.png)
*PBR rendering test.*


## License

This project is licensed under the GNU General Public License version 3 (GPL3).

Copyright (c) 2025-2026 Kadir Aydın.


## QAOS

Built with ❤️ by the open source community.
