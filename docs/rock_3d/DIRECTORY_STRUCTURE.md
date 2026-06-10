# Rock 3D — Estrutura de Diretórios

```
3d-game-engine/
├── Cargo.toml                          # + bin rock-3d
├── docs/
│   ├── ENGINE_ANALYSIS.md
│   └── rock_3d/
│       ├── GDD.md
│       ├── ARCHITECTURE.md
│       ├── DIRECTORY_STRUCTURE.md      # este arquivo
│       ├── SYSTEMS.md
│       ├── FLOWCHART.md
│       └── ROADMAP.md
├── assets/
│   ├── textures/                       # reutilizado
│   ├── models/                         # rochas glTF reutilizadas
│   └── rock_3d/                        # assets futuros do jogo
│       ├── targets/
│       └── maps/
├── saves/
│   ├── world.json                      # Desert Shooter
│   └── rock_3d/
│       ├── profile.json
│       ├── daily.json
│       └── replays/
└── src/
    ├── lib.rs                          # + pub mod core; pub mod games;
    ├── main.rs                         # Desert Shooter (inalterado)
    ├── bin/
    │   └── rock_3d.rs                  # Entry point Rock 3D
    ├── core/
    │   ├── mod.rs
    │   ├── game_plugin.rs
    │   ├── ecs/
    │   │   ├── mod.rs
    │   │   ├── entity.rs
    │   │   ├── component.rs
    │   │   └── world.rs
    │   └── physics/
    │       ├── mod.rs
    │       ├── constants.rs
    │       ├── rigid_body.rs
    │       └── collision.rs
    ├── games/
    │   ├── mod.rs
    │   └── rock_3d/
    │       ├── mod.rs
    │       ├── app.rs
    │       ├── state.rs
    │       ├── throw.rs
    │       ├── stones/
    │       │   ├── mod.rs
    │       │   └── types.rs
    │       ├── targets/
    │       │   ├── mod.rs
    │       │   ├── types.rs
    │       │   └── bosses.rs
    │       ├── physics/
    │       │   ├── mod.rs
    │       │   └── rock_sim.rs
    │       ├── modes/
    │       │   ├── mod.rs
    │       │   ├── arcade.rs
    │       │   ├── precision.rs
    │       │   ├── distance.rs
    │       │   ├── survival.rs
    │       │   ├── daily.rs
    │       │   └── local_mp.rs
    │       ├── progression/
    │       │   ├── mod.rs
    │       │   ├── xp.rs
    │       │   ├── skills.rs
    │       │   └── unlocks.rs
    │       ├── weather/
    │       │   └── mod.rs
    │       ├── scoring/
    │       │   └── mod.rs
    │       ├── replay/
    │       │   └── mod.rs
    │       ├── ai/
    │       │   ├── mod.rs
    │       │   └── states.rs
    │       ├── procedural/
    │       │   └── mod.rs
    │       ├── maps/
    │       │   └── mod.rs
    │       ├── ui/
    │       │   ├── mod.rs
    │       │   └── hud.rs
    │       ├── audio/
    │       │   └── mod.rs
    │       └── save/
    │           └── mod.rs
    ├── engine/                         # existente
    ├── graphics/                       # existente
    ├── assets/                         # existente
    ├── audio/                          # existente
    ├── math/                           # existente
    └── game/                           # Desert Shooter (legado)
```

## Convenções

- Cada submódulo expõe API pública via `mod.rs`
- Testes unitários no mesmo arquivo (`#[cfg(test)]`) ou `tests/rock_3d/`
- Configuração de balanceamento em constantes no topo de cada módulo
- Sem dependência circular: `app` → `state` → subsystems → `core`
