/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once


namespace mochi {

  // Engine
  struct Engine;
  

  // SubSystems
  namespace sys {
    struct RenderingSystem;
    struct DisplaySystem;
    struct SceneSystem;
    struct PhysicSystem;
    struct AudioSystem;
    struct PluginSystem;
  }

  // Audio Hardware Interface
  namespace ahi {

  }

  // Render Hardware Interface
  namespace rhi {
    // Manager
    struct Device;
    struct Allocator;
    struct Uploader;

    // Utility
    struct PipelineCacheUtility;
    struct ShaderCacheUtility;
    struct ShaderCompileUtility;
    
    // Comp
    struct Shader;
    struct PipelineMeta;
    struct Pipeline;
    struct Buffer;
    struct Image;
    struct Image2;
    struct ImageView;
    struct ImageView2;
    struct Sampler;
    struct Sampler2;
    struct RenderTarget;
    struct Command;
    struct Synchronizer;
    
    
    struct CommandManager;
    struct SwapchainManager;
    struct ResourceManager;
  }

  // Utility
  namespace utility {
    struct MaterialUtility;
  }

  // Entity Component System
  namespace ecs {
    struct Camera;
    struct Hierarchy;
    struct Mesh;
    struct MultiMesh;
    struct PointLight;
    struct Skeleton;
    struct Transform;
  }

  // Asset
  namespace asset {
    struct Mesh;
    struct Texture2;
    struct Material;
  }

}
