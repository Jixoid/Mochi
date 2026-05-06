## Mochi

A minimalist Vulkan-based graphics engine and framework written in C++.
QAOS focuses on a clean, layered architecture to simplify low-level graphics programming.

Other languages: [tr](Readme.tr.md)


## Architecture

The project is divided into three core layers:

1. **RHI (Render Hardware Interface):** Low-level GPU resource management.
2. **World:** Scene objects including Nodes, Cameras, Lights, and Visuals.
3. **Asset:** High-level data structures like Meshes.


## Visuals

![Suzanne 3D Model](/images/Suzanne.png)
*PBR rendering test.*


## Building

The project uses CMake for build configuration.
You can also compile it simply as follows:

```bash
scripts/configure.sh
scripts/build.sh
```


## License

This project is licensed under the GNU General Public License version 3 (GPL3). See the LICENSE file for details.

Copyright (c) 2025-2026 Kadir Aydın.


## QAOS

Built with ❤️ by the open source community.
