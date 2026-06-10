# Rock 3D — Roadmap de Desenvolvimento

## Fase 0 — Fundação (Semana 1–2) ✅

- [x] Análise da engine e gaps
- [x] Documentação GDD, arquitetura, sistemas
- [x] Módulo `core/` com ECS e física SI
- [x] Estrutura `games/rock_3d/` com todos os módulos
- [x] Binário `rock-3d` jogável (MVP)

## MVP — Versão 0.1 (Semana 3–4)

**Objetivo:** Provar o core loop — mirar, arremessar, acertar, pontuar.

| Feature | Critério de aceite |
|---------|-------------------|
| Arremesso | Força variável, ângulos H/V, spin lateral/top/back |
| Física | Gravidade, arrasto, Magnus, vento, colisão terreno |
| Pedras | Pequena, Média, Grande funcionais |
| Alvos | Placas e latas estáticas |
| Mapa | Pedreira (terreno procedural) |
| HUD | Força, ângulo, vento, score, combo |
| Pontuação | Base + distância + combo |
| Clima | Vento dinâmico básico |

**Não inclui:** progressão, replay, AI, chefes, multiplayer.

```bash
cargo run --bin rock-3d
```

## Early Access — Versão 0.5 (Mês 2–3)

**Objetivo:** Conteúdo suficiente para 10+ horas de gameplay.

### Sprint 1 — Conteúdo
- [ ] 4 mapas (Pedreira, Floresta, Deserto, Montanha)
- [ ] Todas as 7 pedras (incl. desbloqueáveis via debug)
- [ ] Alvos móveis (drones, carrinhos)
- [ ] 2 chefes (Robô, Torre)

### Sprint 2 — Modos
- [ ] Arcade com 15 fases
- [ ] Precisão e Distância
- [ ] Sobrevivência (ondas)
- [ ] Desafio Diário procedural

### Sprint 3 — Progressão
- [ ] Sistema XP completo
- [ ] Árvore de habilidades (3 ramos)
- [ ] Save/load profile
- [ ] Desbloqueios por nível

### Sprint 4 — Polish
- [ ] IA modular (patrulha, evasão, cobertura)
- [ ] Clima completo (chuva, neblina, tempestade)
- [ ] Replay básico (gravar + playback câmera)
- [ ] SFX (assobio, impacto, ricochete)
- [ ] Partículas de impacto por material

## Versão Final — 1.0 (Mês 4–6)

**Objetivo:** Jogo completo competitivo e rejogável.

### Conteúdo
- [ ] 6 mapas com gameplay único
- [ ] 3 chefes com weak points
- [ ] 30+ fases Arcade
- [ ] Multiplayer local turn-based (2–4 jogadores)

### Sistemas
- [ ] Replay cinematográfico (slow-mo, câmeras)
- [ ] Motion blur na pedra
- [ ] Destruição de estruturas em cadeia
- [ ] Ranking desafio diário (local, futuro online)
- [ ] Skins e efeitos visuais desbloqueáveis

### Qualidade
- [ ] Balanceamento completo (playtesting 50+ sessões)
- [ ] Tutorial interativo
- [ ] Menu principal + settings
- [ ] Suporte gamepad
- [ ] Testes automatizados física + scoring

## Cronograma Visual

```
MVP          Early Access                    Final
|----|----|----|----|----|----|----|----|----|----|----|----|
 W1   W2   W3   W4   M2        M3        M4        M5   M6
[Foundation][MVP playable][Content][Modes][Progress][Polish][1.0]
```

## Métricas por Milestone

| Milestone | Arremessos testáveis | Mapas | Modos | Pedras | Alvos |
|-----------|---------------------|-------|-------|--------|-------|
| MVP | ∞ | 1 | 1 | 3 | 5 tipos |
| EA 0.5 | ∞ | 4 | 5 | 7 | 12 tipos |
| 1.0 | ∞ | 6 | 6 | 7+skins | 20+ tipos |

## Riscos e Mitigações

| Risco | Mitigação |
|-------|-----------|
| Física instável em edge cases | Fixed timestep 120Hz, testes unitários |
| Escopo excessivo | MVP estrito, EA incremental |
| HUD acoplado ao Desert Shooter | RockHud separado, futuro trait HudRenderer |
| Vulkan sem features | Rock 3D target OpenGL primeiro |
