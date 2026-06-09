# Desert Shooter Engine — Renderização Nativa

Game engine 3D **sem frameworks** (sem Bevy, Unity, etc.). A renderização é implementada **do zero** com três backends gráficos:

| Backend | API | Plataforma |
|---------|-----|------------|
| **OpenGL** | OpenGL 3.3 Core | Windows, Linux, macOS |
| **Vulkan** | Vulkan 1.2 (`ash`) | Windows, Linux |
| **DirectX** | Direct3D 11 (`windows`) | Windows |

## Arquitetura

```text
┌──────────────────────────────────────────────────────────┐
│  EngineApp              loop principal (winit)             │
├──────────────────────────────────────────────────────────┤
│  GfxBackend (trait)     interface comum aos 3 backends   │
│    ├─ OpenGLRenderer    VAO/VBO + shaders GLSL           │
│    ├─ VulkanRenderer    instance/device/pipeline (ash)   │
│    └─ DirectX11Renderer  D3D11 + shaders HLSL            │
├──────────────────────────────────────────────────────────┤
│  SceneBuilder           constrói mapa + alvos              │
│  GameWorld              jogador FPS, tiro, pontuação     │
└──────────────────────────────────────────────────────────┘
```

## Requisitos

- [Rust](https://rustup.rs/) 1.70+
- **OpenGL**: drivers com OpenGL 3.3+
- **Vulkan**: [Vulkan SDK](https://vulkan.lunarg.com/) + drivers + **CMake** (compila shaders via `shaderc`)
- **DirectX**: Windows 10+ (DirectX 11 nativo)

## Como executar

```bash
# OpenGL (padrão — funciona em qualquer plataforma)
cargo run

# Vulkan
cargo run --features vulkan --no-default-features
set GFX_BACKEND=vulkan    # Windows CMD
$env:GFX_BACKEND="vulkan"  # PowerShell

# DirectX 11 (só Windows)
cargo run --features directx --no-default-features
set GFX_BACKEND=directx

# Compilar com todos os backends
cargo run --features all-backends
```

## Controles

| Ação | Função |
|------|--------|
| **WASD** | Mover |
| **Shift** | Correr |
| **Mouse** | Olhar (clique para travar) |
| **Clique** | Atirar |
| **ESC** | Liberar mouse |

## Adicionar alvos na cena

```rust
use desert_shooter_engine::prelude::*;

let scene = SceneBuilder::new()
    .with_desert_map()
    .with_player_spawn(Vec3::new(0.0, 1.7, 8.0))
    .add_target(Vec3::new(10.0, 0.0, -20.0))      // 100 pts
    .add_target_at(5.0, 0.0, -15.0, 250);          // 250 pts

EngineApp::new()
    .with_scene(scene)
    .run();
```

## Estrutura do código (didática)

```
src/
├── main.rs                    # Demo do jogo
├── engine/app.rs              # Loop principal
├── graphics/
│   ├── backend.rs             # Trait GfxBackend
│   ├── shaders.rs             # GLSL + HLSL (código da GPU)
│   ├── primitives.rs          # Gera meshes na CPU
│   ├── opengl/renderer.rs     # OpenGL do zero
│   ├── vulkan/renderer.rs     # Vulkan do zero (ash)
│   └── directx/renderer.rs    # DirectX 11 do zero
└── game/
    ├── scene.rs               # SceneBuilder
    ├── desert.rs              # Mapa do deserto
    ├── player.rs              # Câmera FPS
    └── shooting.rs            # Raycast raio-esfera
```

## O que cada backend faz internamente

### OpenGL (`src/graphics/opengl/`)
1. Cria contexto com **glutin**
2. Compila shaders **GLSL** em runtime
3. Envia vértices via **VBO/VAO/EBO**
4. `glDrawElements` a cada objeto
5. `swap_buffers` para exibir

### Vulkan (`src/graphics/vulkan/`)
1. Cria **Instance** e **Surface**
2. Seleciona GPU e cria **Device**
3. **Swapchain** para double-buffering
4. **Render Pass** + **Pipeline** gráfico
5. Compila GLSL → SPIR-V via **shaderc**
6. **Command Buffers** para draw calls

### DirectX 11 (`src/graphics/directx/`)
1. Cria **ID3D11Device** + **IDXGISwapChain**
2. Compila shaders **HLSL** via `D3DCompile`
3. **Vertex/Index Buffers** na GPU
4. **Constant Buffer** para matriz MVP
5. `DrawIndexed` + `Present`

## Licença

MIT OR Apache-2.0
