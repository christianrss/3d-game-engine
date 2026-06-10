# Rock 3D — Game Design Document

**Versão:** 1.0  
**Gênero:** Competitive Physics Arcade  
**Plataforma:** PC (Windows/Linux/macOS)  
**Engine:** Desert Shooter Engine (Rust)  
**Público:** Casual a hardcore — fácil de aprender, difícil de dominar

---

## 1. Visão

Rock 3D é um jogo competitivo de arremesso de pedras em ambientes 3D. O jogador mira, escolhe a pedra certa, calcula trajetória considerando vento, gravidade, distância, altura e obstáculos, e tenta destruir diversos tipos de alvos.

**Pilares de design:**
- **Skill expression** — maestria em física e timing
- **Variedade** — pedras, alvos, mapas, clima
- **Progressão** — desbloqueios e árvore de habilidades
- **Rejogabilidade** — modos, desafio diário, multiplayer local

**Referências:** Angry Birds (destruição), Wii Sports (acessibilidade), Golf (precisão/distância), bow games (vento/trajetória), physics sandbox (emergência), arcade skill games (combos).

---

## 2. Loop Principal

```
Escolher pedra → Mirar (ângulo H/V, força, spin) → Arremessar
    → Observar física → Impacto/destruição → Pontuação/combo
    → Próximo arremesso ou fim da rodada → XP/desbloqueios
```

**Sessão típica (Arcade):** 3–5 minutos por fase, 10–20 arremessos.

---

## 3. Mecânicas de Arremesso

| Parâmetro | Input | Efeito |
|-----------|-------|--------|
| Força | Segurar LMB (charge) | Velocidade inicial (5–45 m/s) |
| Ângulo horizontal | Mouse X / A-D | Rotação yaw do lançamento |
| Ângulo vertical | Mouse Y / W-S | Pitch (-15° a +75°) |
| Spin lateral | Q/E ou scroll | Curva horizontal (efeito Magnus) |
| Spin superior | R | Mais lift, menos queda |
| Spin inferior | F | Mais queda, menos lift |

**Dispersão:** base + modificadores de pedra + habilidades + vento + skill tree.

**Preview de trajetória:** desbloqueável via habilidade "Visão de Trajetória" (pontos fantasma).

---

## 4. Física (unidades SI)

| Propriedade | Valor / Comportamento |
|-------------|----------------------|
| Gravidade | 9.81 m/s² |
| Massa pedra | 0.15–2.5 kg |
| Raio | 0.04–0.18 m |
| Arrasto | Cd × ½ρv²A (ρ ar = 1.225 kg/m³) |
| Magnus | F = S × (ω × v) |
| Atrito solo | μ = 0.3–0.8 conforme superfície |
| Restituição | e = 0.1–0.85 conforme material |
| Transferência energia | conservação linear + perda inelástica |

**Colisões:** esfera-esfera, esfera-AABB, esfera-plano (terreno). Ricochetes calculam reflexão com perda de energia.

---

## 5. Sistema de Pedras

| Pedra | Peso (kg) | Raio (m) | Cd | Dano | Notas |
|-------|-----------|----------|-----|------|-------|
| Pequena | 0.15 | 0.04 | 0.42 | 1× | Rápida, precisa |
| Média | 0.45 | 0.07 | 0.40 | 2× | Balanceada (padrão) |
| Grande | 1.2 | 0.12 | 0.38 | 4× | Lenta, alto impacto |
| Lisa | 0.35 | 0.06 | 0.28 | 1.5× | Aerodinâmica |
| Irregular | 0.55 | 0.08 | 0.55 | 2.5× | +15% dispersão, spin imprevisível |
| Metálica | 2.0 | 0.10 | 0.25 | 5× | Desbloqueável Lv.15 |
| Explosiva | 0.40 | 0.07 | 0.40 | 3× + AoE | Desbloqueável Lv.25 |

**Resistência:** pedras não quebram exceto Explosiva (consumível).

---

## 6. Sistema de Alvos

### Estáticos
- **Placas** — 50 pts, fácil
- **Latas** — 75 pts, empilháveis
- **Garrafas** — 100 pts, frágeis (alta restituição)
- **Sinos** — 150 pts, som especial ao acertar

### Móveis
- **Drones** — 200 pts, patrulha aérea
- **Carrinhos** — 175 pts, trilho linear
- **Plataformas** — alvo montado, oscila

### Inteligentes (NPCs)
- Estados: Idle → Patrulha → Alerta → Evasão → Cobertura
- Reagem a near-miss e som de impacto
- 250–400 pts

### Chefes
| Chefe | HP | Pontos Fracos | Recompensa |
|-------|-----|---------------|------------|
| Robô Gigante | 500 | Joelhos, antena | 2000 pts + pedra metálica |
| Torre Blindada | 800 | Portas, janelas | 3000 pts + skin |
| Drone Colossal | 600 | Motores, núcleo | 2500 pts + habilidade |

---

## 7. Modos de Jogo

| Modo | Descrição | Duração |
|------|-----------|---------|
| **Arcade** | Fases com alvos fixos, 3 estrelas por precisão/tempo/arremessos | 3–5 min |
| **Precisão** | Alvos pequenos, vento variável | 5 min |
| **Distância** | Alvos 50–200 m, contagem de arremessos | Ilimitado |
| **Sobrevivência** | Ondas infinitas, dificuldade crescente | Até falhar |
| **Desafio Diário** | Seed procedural, ranking global | 1 tentativa/dia |
| **Multiplayer Local** | Turnos alternados, mesmo cenário, maior pontuação vence | 10–15 min |

---

## 8. Progressão

### XP
- Acerto: 10–500 XP (conforme dificuldade)
- Combo ×1.5 por acerto consecutivo (máx ×5)
- Chefe derrotado: 1000 XP
- Primeira estrela em fase: 200 XP

### Desbloqueáveis (níveis)
| Nível | Desbloqueio |
|-------|-------------|
| 3 | Pedra Lisa |
| 5 | Mapa Floresta |
| 8 | Luva de Precisão |
| 10 | Pedra Irregular |
| 12 | Mapa Deserto |
| 15 | Pedra Metálica |
| 18 | Mapa Montanha |
| 20 | Skin Dourada |
| 22 | Mapa Cidade |
| 25 | Pedra Explosiva |
| 30 | Mapa Futurista |

### Árvore de Habilidades (3 ramos)

**Força:** Braço Forte (+10% força) → Arremesso Duplo → Rajada  
**Precisão:** Olho de Águia (-20% dispersão) → Visão Trajetória → Mira Lenta  
**Técnica:** Vento Calmo (-30% vento) → Ricochete Controlado → Spin Mestre

Cada nó custa 1–3 pontos de habilidade (ganhos a cada 5 níveis).

---

## 9. Ambientes

| Mapa | Gameplay | Clima típico |
|------|----------|--------------|
| Pedreira | Alvos em prateleiras, ricochetes em rocha | Vento moderado |
| Floresta | Obstáculos (árvores), alvos parcialmente ocultos | Neblina, chuva |
| Deserto | Vento forte, calor (arrasto reduzido) | Tempestade de areia |
| Montanha | Altitude (gravidade -2%), alvos elevados | Neve, vento forte |
| Cidade Abandonada | Estruturas colapsáveis, NPCs em cobertura | Chuva |
| Instalação Futurista | Escudos energéticos, drones | Tempestade elétrica |

---

## 10. Sistema Climático

| Clima | Efeito na física | Efeito visual |
|-------|------------------|---------------|
| Vento dinâmico | Força lateral proporcional a v² | Partículas, bandeira |
| Chuva | +20% arrasto, -10% atrito | Gotas, poças |
| Neblina | -visibilidade (sem HUD distância longe) | Fog density |
| Tempestade | Vento errático ±50%, raios | Flash, trovão |
| Temperatura | Frio: +densidade ar; Calor: -densidade | Cor do céu |

---

## 11. Pontuação

```
Pontos = base_alvo
       × multiplicador_distância (1 + dist/50)
       × multiplicador_velocidade (1 + |v|/20)
       × combo_streak
       + bônus_ricochete (50 por ricochete antes do acerto)
       + bônus_tempo (tempo_restante × 2)
```

**Estrelas (Arcade):** ★ precisão >70%, ★★ tempo <60s, ★★★ ≤5 arremessos.

---

## 12. Replay

Grava por frame: posição pedra, velocidade, spin, eventos de colisão, pontuação acumulada.

Playback: câmera livre, seguir pedra, câmera impacto (slow-mo 0.25×).

---

## 13. Áudio

| Evento | Som |
|--------|-----|
| Carregar força | Tensão crescente (pitch↑) |
| Arremesso | Whoosh (velocidade-dependente) |
| Assobio no ar | Proporcional a \|v\| |
| Impacto | Material-dependente (metal, madeira, vidro) |
| Ricochete | Tom metálico + reverb |
| Destruição | Crash + fragmentos |
| Ambiente | Loop por mapa + clima overlay |

---

## 14. Interface (HUD)

```
┌──────────────────────────────────────────────────┐
│ [Pedra: Média]  XP: 1240  Lv.8    Score: 1850  │
│                                                  │
│              ◉ mira dinâmica                     │
│                                                  │
│ Força ████████░░ 42%    Combo ×3                 │
│ Ângulo H: 23°  V: 45°   Spin: L+2 T-1           │
│ Vento → 4.2 m/s  NE    Distância: 38m           │
│ Arremessos: 3/8         Tempo: 1:24              │
└──────────────────────────────────────────────────┘
```

---

## 15. Controles

| Ação | Teclado/Mouse |
|------|---------------|
| Mirar | Mouse |
| Carregar força | Segurar LMB |
| Arremessar | Soltar LMB |
| Spin lateral | Q / E |
| Spin top/back | R / F |
| Trocar pedra | 1–7 |
| Preview trajetória | Tab (se desbloqueado) |
| Pausa | Esc |
| Replay último | V |

---

## 16. Métricas de Sucesso

- **MVP:** arremesso funcional, 1 mapa, 3 pedras, alvos estáticos, pontuação
- **Early Access:** 4 modos, 4 mapas, progressão, clima, 2 chefes
- **Final:** todos modos, 6 mapas, multiplayer, replay, árvore completa
