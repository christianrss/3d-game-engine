# Análise da Engine — Estado Atual e Organização Multi-Jogo

## Resumo Executivo

A **Desert Shooter Engine** é uma engine 3D didática em Rust com renderização nativa (OpenGL completo, Vulkan/DirectX básicos). Atualmente funciona como **engine monolítica acoplada a um único jogo** (Mega Deserto / Desert Shooter). Para suportar múltiplos tipos de jogos — incluindo Rock 3D — é necessário extrair camadas reutilizáveis e adotar arquitetura modular com ECS.

---

## O Que Já Existe

| Sistema | Status | Módulo | Reutilizável? |
|---------|--------|--------|---------------|
| Renderização OpenGL | Completo (PBR, sombras, bloom, água, HUD) | `graphics/opengl/` | Sim |
| Renderização Vulkan/DX11 | Básico (mesh + luz) | `graphics/vulkan/`, `directx/` | Parcial |
| Loop principal (winit) | Completo | `engine/app.rs` | Precisa abstração |
| Câmera FPS | Completo | `graphics/camera.rs` | Sim |
| Input | Completo | `game/input.rs` | Sim (generalizar) |
| Áudio procedural | Básico | `audio/mod.rs` | Sim (expandir) |
| Assets (glTF, OBJ, procedural) | Completo | `assets/` | Sim |
| Terreno procedural | Completo | `assets/terrain.rs` | Sim (parametrizar) |
| Física balística | Básico | `game/projectile.rs` | Base para Rock 3D |
| Colisão esfera/AABB | Básico | `game/physics.rs` | Base |
| Partículas | Completo | `game/particles.rs` | Sim |
| Save JSON | Completo | `game/persistence.rs` | Sim (generalizar) |
| Event log | Básico | `game/events.rs` | Base para replay |
| Networking UDP | Protótipo | `game/net.rs` | Parcial |
| Day/Night | Completo | `game/daynight.rs` | Sim |
| Score | Básico | `game/score.rs` | Sim |

---

## O Que Falta para Engine Genérica

### Crítico (bloqueia multi-jogo)

1. **ECS (Entity-Component-System)** — hoje usa structs monolíticos (`GameWorld`, `Ecosystem`, `WorldSimulation`)
2. **GamePlugin trait** — abstração para registrar jogos sem modificar `EngineApp`
3. **Física reutilizável** — rigid body, impulso, spin, Magnus, atrito, restituição
4. **Scene graph** — hierarquia de entidades com transform
5. **UI framework** — HUD é hardcoded para Desert Shooter

### Importante (qualidade AAA)

6. **Sistema de áudio espacial** — hoje só synth procedural
7. **Replay system** — event log existe mas sem playback
8. **Procedural generation framework** — noise existe, falta orquestração
9. **AI state machine modular** — ecosystem tem AI mas acoplada
10. **Backend parity** — Vulkan/DX sem PBR/HUD/partículas

### Desejável (polish)

11. Gamepad, localização, settings screen
12. Asset hot-reload, async loading
13. CI/CD, testes automatizados, `examples/`
14. Broad-phase collision (BVH), mesh raycast

---

## Arquitetura Proposta — Multi-Jogo

```
┌─────────────────────────────────────────────────────────────┐
│                    APPLICATION LAYER                         │
│  bin/desert-shooter    bin/rock-3d    bin/[future-game]     │
├─────────────────────────────────────────────────────────────┤
│                    GAMES LAYER                               │
│  games/desert_shooter/    games/rock_3d/                    │
│  (migração gradual)       (novo jogo completo)              │
├─────────────────────────────────────────────────────────────┤
│                    CORE LAYER (novo)                         │
│  core/ecs/    core/physics/    core/game_plugin.rs          │
│  core/input/  core/save/       core/replay/                 │
├─────────────────────────────────────────────────────────────┤
│                    ENGINE LAYER (existente)                  │
│  engine/  graphics/  assets/  audio/  math/                │
└─────────────────────────────────────────────────────────────┘
```

### Princípios

- **Engine** = renderização, assets, áudio base, math — sem lógica de jogo
- **Core** = ECS, física, save, replay, input abstrato — compartilhado entre jogos
- **Games** = regras, mecânicas, conteúdo — cada jogo é um módulo isolado
- **Bins** = entry points finos que instanciam o jogo desejado

### Migração do Desert Shooter

O código atual em `src/game/` permanece funcional. A migração para `games/desert_shooter/` é incremental — não quebra o binário existente.

---

## Matriz de Compatibilidade por Gênero

| Gênero | ECS | Física | Render | Input | Save | Net |
|--------|-----|--------|--------|-------|------|-----|
| FPS (Desert Shooter) | Opcional | Básica | Full | FPS | Sim | Parcial |
| Arremesso (Rock 3D) | Sim | Avançada | Full | Aim+Throw | Sim | Turn-based |
| Puzzle 3D | Sim | Básica | Full | Point+Click | Sim | Não |
| Sandbox | Sim | Média | Full | FPS+Build | Sim | Sim |
| Racing | Sim | Avançada | Full | Vehicle | Sim | Sim |

---

## Próximos Passos da Engine

1. ✅ Criar `src/core/` com ECS e física
2. ✅ Criar `src/games/rock_3d/` como primeiro jogo modular
3. ⬜ Extrair `GamePlugin` e permitir `EngineApp::with_game(plugin)`
4. ⬜ Generalizar HUD via trait `HudRenderer`
5. ⬜ Migrar Desert Shooter para `games/desert_shooter/`
6. ⬜ Adicionar `examples/` com cenas mínimas
