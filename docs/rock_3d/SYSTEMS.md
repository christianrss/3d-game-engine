# Rock 3D — Lista de Sistemas

## Engine (reutilizados)

| ID | Sistema | Módulo | Status |
|----|---------|--------|--------|
| E01 | Renderização OpenGL PBR | `graphics/opengl/` | ✅ Produção |
| E02 | Câmera 3D | `graphics/camera.rs` | ✅ |
| E03 | Asset loading glTF/OBJ | `assets/` | ✅ |
| E04 | Terreno procedural | `assets/terrain.rs` | ✅ |
| E05 | Partículas CPU | `game/particles.rs` | ✅ |
| E06 | Áudio base rodio | `audio/mod.rs` | ✅ |
| E07 | Day/Night | `game/daynight.rs` | ✅ |
| E08 | Input winit | `game/input.rs` | ✅ |

## Core (novos)

| ID | Sistema | Módulo | Status |
|----|---------|--------|--------|
| C01 | ECS World | `core/ecs/world.rs` | ✅ Inicial |
| C02 | Entity/Component storage | `core/ecs/` | ✅ Inicial |
| C03 | RigidBody SI | `core/physics/rigid_body.rs` | ✅ Inicial |
| C04 | Colisão esfera/plano/AABB | `core/physics/collision.rs` | ✅ Inicial |
| C05 | Constantes físicas | `core/physics/constants.rs` | ✅ |
| C06 | GamePlugin trait | `core/game_plugin.rs` | ✅ Inicial |

## Rock 3D — Gameplay

| ID | Sistema | Módulo | Status |
|----|---------|--------|--------|
| G01 | Throw controller | `throw.rs` | ✅ MVP |
| G02 | Stone types | `stones/types.rs` | ✅ |
| G03 | Rock flight sim | `physics/rock_sim.rs` | ✅ MVP |
| G04 | Target registry | `targets/types.rs` | ✅ |
| G05 | Boss encounters | `targets/bosses.rs` | ✅ Stub |
| G06 | Scoring + combos | `scoring/mod.rs` | ✅ |
| G07 | Game modes | `modes/` | ✅ Stub |
| G08 | Map definitions | `maps/mod.rs` | ✅ |
| G09 | Weather/climate | `weather/mod.rs` | ✅ |
| G10 | AI state machine | `ai/states.rs` | ✅ |
| G11 | Procedural layout | `procedural/mod.rs` | ✅ |
| G12 | XP + levels | `progression/xp.rs` | ✅ |
| G13 | Skill tree | `progression/skills.rs` | ✅ |
| G14 | Unlocks | `progression/unlocks.rs` | ✅ |
| G15 | Replay recorder | `replay/mod.rs` | ✅ |
| G16 | Profile save | `save/mod.rs` | ✅ |
| G17 | Rock HUD | `ui/hud.rs` | ✅ |
| G18 | Game audio | `audio/mod.rs` | ✅ |
| G19 | Game state machine | `state.rs` | ✅ |
| G20 | Application loop | `app.rs` | ✅ MVP |

## Pendentes (pós-MVP)

| ID | Sistema | Prioridade |
|----|---------|------------|
| P01 | Multiplayer local turn-based | Alta |
| P02 | Câmera replay cinematográfica | Média |
| P03 | Motion blur pedra | Média |
| P04 | Destruição procedural de estruturas | Alta |
| P05 | Áudio espacial 3D | Média |
| P06 | Ranking desafio diário online | Baixa |
| P07 | 6 mapas completos com arte | Alta |
| P08 | Chefes com múltiplos weak points | Alta |
