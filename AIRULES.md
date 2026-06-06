## 1. Project Overview & Core Philosophy

**Mochi** is a high-performance graphics and game engine built from the ground up using **Modern C++**. It is designed as a modern framework tailored specifically for cutting-edge **Vulkan API** features, moving away from legacy graphics paradigms.

### Core Architectural Pillars

- **Uncompromising Performance:** Every design choice, data structure, and algorithm must prioritize execution speed, low memory overhead, and cache locality.
    
- **Modern Vulkan First:** The engine is built around modern Vulkan extensions and features, explicitly leveraging:
    
    - **Buffer Device Address (BDA):** For direct GPU memory pointer access, eliminating binding overhead.
        
    - **Bindless Textures / Resources:** Utilizing large descriptor sets to dynamically index resources inside shaders, completely removing traditional binding bottlenecks.
        
- **Data-Oriented Design (DOD):** Memory layouts should favor contiguous allocations and cache-friendly structures to feed the GPU efficiently.
    

### AI Implementation Rules

- **Rule 1 (C++ Standard):** Always write clean, idiomatic Modern C++. Use modern abstractions but ensure they do not introduce hidden performance or memory penalties (avoid unnecessary allocations, `std::shared_ptr` abuse, or heavy runtime RTTI where flat structures or explicit types fit better).
    
- **Rule 2 (Vulkan Paradigm):** Do not suggest legacy Vulkan workflows (e.g., old-school descriptor set switching per draw call). Always design APIs and abstractions around bindless architectures and direct memory addressing (BDA).
    
- **Rule 3 (Performance First):** When generating or refactoring code, optimize for memory alignment, minimize cache misses, and avoid redundant state changes.


## 2. Tech Stack, Build System & Code Structure

### Build & Compilation Environment

- **Language Standard:** **C++23** is strictly enforced. Leverage modern language features (e.g., modules where appropriate, advanced metaprogramming, concepts, and standard library updates) but always balance with the performance philosophy.
    
- **Build System:** **CMake 4.3+** target. The build configuration relies on modern CMake targets and explicit dependency linking.
    

### Third-Party Dependencies (`/thirdparty/`)

All external dependencies reside in `/thirdparty/DEPENDENCY_NAME/`.

- **Management:** CMake manages and fetches specific versions of these libraries directly via Git.
    
- **Structure:** Each dependency folder contains only its dedicated `CMakeLists.txt` and its `LICENSE` file.
    
- **Active Stack:**
    
    - `cgltf`: GLTF loading utility.
        
    - `entt`: Entity Component System (ECS) framework.
        
    - `stb`: Single-file public domain libraries (image loading, etc.).
        
    - `vma`: Vulkan Memory Allocator.
        

### Internal Engine Submodules (`/mochi/`)

The core engine code is divided into isolated submodules under the `/mochi/` directory (e.g., `/mochi/math`, `/mochi/rhi`).

- **File Extensions:** Always use `.hh` for header files and `.cc` for source files.
    
- **Umbrella Headers:** Every submodule must provide a single master header file named exactly after the submodule (e.g., `/mochi/math/math.hh`). This umbrella header is responsible for including all relevant public headers within that submodule, allowing external modules to include the entire submodule with a single statement.
    
- **Architectural Boundaries:** Submodules must communicate strictly according to the defined dependency graph boundaries. Upward or unauthorized cross-layer includes are strictly prohibited.
    

### AI Implementation Rules

- **Rule 1 (File Creation):** When writing new modules or components, always separate declarations into `.hh` and implementations into `.cc`. Do not use `.h` or `.cpp`.
    
- **Rule 2 (Inclusion Pattern):** When a submodule needs to consume another submodule, it should ideally include its umbrella header (e.g., `#include <mochi/math/math.hh>`), respecting the structural layout.
    
- **Rule 3 (CMake Management):** Do not manually copy source code files into `/thirdparty/`. Rely entirely on the internal CMake automation scripts to fetch specific Git tags/hashes.


## 3. Strict Syntax & Coding Standards

This section outlines the non-negotiable syntactic, structural, and architectural rules for all source files. AI must follow these patterns perfectly without exception.

### File Header & Boilerplate

Every single C++ source (`.cc`) and header (`.hh`) file must begin exactly with the following license text, followed immediately by `#pragma once` (for headers), include guards, and specific spacing rules.

C++

```
/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/
#pragma once

// <--- EXACTLY 1 BLANK LINE HERE AFTER PRAGMA ONCE
#include <mochi/core/example.hh>     // 1st: Project Internal Files
#include <entt/entt.hpp>             // 2nd: Third-Party Dependencies
#include <expected>                  // 3rd: C++ Standard Library Headers



// <--- EXACTLY 3 BLANK LINES HERE AFTER ALL INCLUDES
namespace Mochi
{ // <--- Opening brace MUST be on the next line
```

### Type Definitions & Keywords

- **No `class` Keyword:** The `class` keyword is strictly banned. Use `struct` for all type definitions.
    
- **No Raw `enum`:** Traditional enums are strictly banned. Always use `enum struct` (or `enum class`).
    

### Brace (`{`) Placement Rules

- **Namespaces:** Always place the opening brace on the next line.
    
- **Data-Only Structs (POD/Aggregates):** If a struct contains _only data_ and no functions, the opening brace must be on the **same line** as the struct declaration.
    
- **Structs with Functions:** If a struct contains any member functions, the opening brace must be on the **next line**.
    
- **Functions in Header Files (`.hh`):** The opening brace must be on the **same line** as the function signature.
    
- **Functions in Source Files (`.cc`):** The opening brace must be on the **next line**.
    
    - _Exception:_ Temporary, localized inline functions within `.cc` files can have the brace on the same line.
        

### Naming Conventions

- **Type Names:** `CamelCase` (e.g., `RenderContext`).
    
- **Parameter Names:** `CamelCase` (e.g., `bufferSize`).
    
- **Enum Values:** `PascalCase` (e.g., `VertexShader`).
    
- **Private Members:** Must start with an `m_` prefix followed by CamelCase (e.g., `m_deviceMemory`).
    
- **Private Static Functions:** Must start with an `f_` prefix followed by CamelCase (e.g., `f_createInternalBuffer`).
    

### Error Handling Paradigm

- **Expected/Recoverable Errors:** Always use `std::expected` to return values or error codes cleanly without performance overhead.
    
- **Exceptional/Fatal Errors:** Use `throw` exclusively for truly exceptional, unrecoverable runtime states.
    

### Struct Grouping & Internal Layout Architecture

Access modifiers (`public`, `private`, `protected`) must be used as strict section dividers to group structural contents.

- **Data-Only Structs:** Do **not** use any access modifiers (like `public:`) if the struct contains no functions.
    
- **Structs with Functions:** Contents must be meticulously organized into the following sections in order. Separate each block with exactly **2 blank lines** (except after the final block).
    

1. **Section 1 (Initialization):** Constructors and Destructors.
    
2. **Section 2 (Factories):** Static instantiation or factory functions.
    
3. **Section 3 (Hidden State):** Private variables prefixed with `m_`.
    
4. **Section 4 (Encapsulation Getters):** Public inline reference-getter functions that expose the private `m_` variables directly via reference, named exactly like the variable but _without_ the `m_` prefix.
    
5. **Section 5+ (Dynamic Logic):** Remaining member functions, arranged dynamically based on context.
    

#### Comprehensive Layout Example (`.hh`)

C++

```
struct CommandBuffer
{
public:
    CommandBuffer();
    ~CommandBuffer();


public:
    static std::expected<CommandBuffer, ErrorCode> CreateManagedBuffer(uint32_t bufferSize);


private:
    VmaAllocation m_bufferAllocation;
    uint64_t m_gpuAddress;


public:
    inline auto& bufferAllocation() { return m_bufferAllocation; }
    inline auto& gpuAddress() { return m_gpuAddress; }


public:
    void RecordSubmit() {
        // Implementation here (brace on same line in header)
    }
};
```

#### Comprehensive Layout Example (`.cc`)

C++

```
std::expected<CommandBuffer, ErrorCode> CommandBuffer::CreateManagedBuffer(uint32_t bufferSize)
{ // <--- Brace on the next line in source files
    // Implementation
}
```


## 4. Submodule Dependency Architecture & Strict Boundaries

The project's internal submodules within `/mochi/` are governed by a strict directed acyclic graph (DAG). High-level layers consume lower-level layers. **Any dependency relationship not explicitly defined below is strictly prohibited.**

### Architectural Layering Reference

1. **Layer 1 (Core Foundations):** `math`, `reader` (Completely independent; zero internal engine dependencies)
    
2. **Layer 2 (Hardware Interface):** `rhi` (Depends strictly on Layer 1)
    
3. **Layer 3 (Resource Management):** `asset` (Depends strictly on Layers 1 and 2)
    
4. **Layer 4 (Systems):** `ecs` (Depends strictly on Layers 1, 2, and 3)
    
5. **Layer 5 (Application/Game Logic):** `module` (Depends strictly on Layers 1, 2, 3, and 4)
    
6. **Layer 6 (Engine Core Aggregator):** `core` (Consumes `rhi`, `ecs`, and `module`)
    

### Immutable Dependency Matrix (Allowed Includes)

- **`math`** $\rightarrow$ No internal dependencies.
    
- **`reader`** $\rightarrow$ No internal dependencies.
    
- **`rhi`** $\rightarrow$ Can only include: `math`.
    
- **`asset`** $\rightarrow$ Can only include: `math`, `reader`, `rhi`.
    
- **`ecs`** $\rightarrow$ Can only include: `math`, `rhi`, `asset`.
    
- **`module`** $\rightarrow$ Can only include: `math`, `rhi`, `asset`, `ecs`.
    
- **`core`** $\rightarrow$ Can only include: `rhi`, `ecs`, `module`.
    

### Dependency Graph Visual (Mermaid)

Kod snippet'i

```
graph TD
    %% Lower Levels (Foundations)
    math["math (Layer 1)"]
    reader["reader (Layer 1)"]
    
    %% Mid Levels
    rhi["rhi (Layer 2)"]
    asset["asset (Layer 3)"]
    ecs["ecs (Layer 4)"]
    module["module (Layer 5)"]
    
    %% Top Level
    core["core (Layer 6)"]

    %% Allowed Directional Dependencies
    math --> rhi
    math --> asset
    math --> ecs
    math --> module
    
    reader --> asset
    
    rhi --> asset
    rhi --> ecs
    rhi --> module
    rhi --> core
    
    asset --> ecs
    asset --> module
    
    ecs --> module
    ecs --> core
    
    module --> core
```

### Absolute Enforcement & Modification Protocol for AI

- **Rule 1 (Zero Tolerance for Unauthorized Includes):** You are strictly forbidden from adding an `#include` directive or linking a submodule if that relationship is not explicitly stated in the matrix above. For example, `ecs` can never include anything from `module` or `core`.
    
- **Rule 2 (Modification Protocol):** If a new feature or refactor heavily implies that a module needs an undocumented dependency (e.g., if `rhi` suddenly needs to know about `ecs`), **you must halt immediately and ask the user for explicit permission.**
    
- **Rule 3 (Cycle and Logic Verification):** Before presenting a dependency change request to the user, you must run a static analysis simulation in your context to ensure the change:
    
    1. Does **not** introduce a circular dependency (e.g., A $\rightarrow$ B $\rightarrow$ A).
        
    2. Does **not** break the logical abstraction layers of a modern graphics framework.
        
- **Rule 4 (Self-Documentation Update):** If the user grants permission and modifies the submodule architecture, your very first task before generating any engine code is to **rewrite and update this section of `AIRULES.md`** to reflect the new graph truth.


## 5. Documentation & Commenting Strategy

To keep the codebase clean and maintain high readability, documentation is strictly separated between interface declarations (`.hh`) and implementation details (`.cc`).

### Doxygen Specifications (Header Files Only - `.hh`)

Doxygen blocks are exclusively reserved for public API declarations inside header files. They must be concise and avoid cluttering the interface.

- **Target Only:** Write Doxygen documentation only for complex or non-obvious member functions.
    
- **Exclusions:** **Do not** write Doxygen comments for constructors, destructors, factory functions, or basic reference-getters. Their purpose is self-explanatory.
    
- **Format:** Limit the `@brief` description to a single sentence if possible. Document only the parameters (`@param`) and the explicit purpose.
    

#### Header Example (`.hh`)

C++

```
struct TextureManager
{
public:
    TextureManager(); // Excluded from Doxygen
    ~TextureManager(); // Excluded from Doxygen

public:
    /**
     * @brief Allocates and binds a bindless texture to the GPU.
     * @param filepath Path to the image asset.
     * @return The descriptor index for bindless access, or an error code.
     */
    std::expected<uint32_t, ErrorCode> LoadBindlessTexture(std::string_view filepath);

private:
    uint32_t m_textureCount;

public:
    inline auto& textureCount() { return m_textureCount; } // Excluded from Doxygen
};
```

### Implementation Comments (Source Files Only - `.cc`)

Code explanations must live strictly inside source files. Comments are divided into two explicit categories based on their syntax.

- **Single-Line Explanations (`//`):** Used to explain non-trivial logic. These comments must be brief, precise, and ideally finish within a single sentence.
    
- **Logical Section Dividers (`///`):** Used exclusively within large or complex functions to partition the execution flow into distinct logical stages.
    
- **Anti-Pattern Warning:** Do not comment on self-explanatory or visually obvious code lines (e.g., avoid `// increment counter` next to `count++;`).
    

#### Source Example (`.cc`)

C++

```
std::expected<uint32_t, ErrorCode> TextureManager::LoadBindlessTexture(std::string_view filepath)
{
    /// Part 1: Resource Loading
    auto imageData = StbLoadImage(filepath);
    if (!imageData) 
    {
        return std::unexpected(ErrorCode::FileNotFound);
    }

    /// Part 2: GPU Allocation
    // Query direct memory addressing capability before allocating VMA buffer
    VkBufferCreateInfo bufferInfo = f_createInternalBufferInfo(imageData.size);
    
    /// Part 3: Bindless Registration
    // Update the global descriptor set index to make the texture bindless
    uint32_t bindlessIndex = f_registerGlobalDescriptor(bufferInfo);

    return bindlessIndex;
}
```
