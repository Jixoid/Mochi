/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once


namespace mochi
{

  struct core;
  

  namespace module {
    struct display;
    struct renderer;
    struct memory;
  }


  namespace rhi {
    struct DeviceManager;
    struct AllocManager;
    struct ShaderManager;
    struct PipelineManager;
    struct TransferManager;
    struct SyncManager;
    struct CommandManager;
    struct SwapchainManager;
    struct ResourceManager;

    struct MaterialManager;

    struct PipelineMeta;
    struct Pipeline;
    
    struct Buffer;
    struct Shader;
    struct Image;
    struct Image2;
  }


  namespace asset {
    struct Mesh;
    struct texture2;
    struct matrial;
  }

}
