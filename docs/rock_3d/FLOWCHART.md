# Rock 3D — Fluxograma dos Sistemas

## Loop Principal do Jogo

```mermaid
flowchart TD
    START([Início]) --> MENU[Menu Principal]
    MENU --> SELECT{Escolher Modo}
    SELECT --> ARCADE[Arcade]
    SELECT --> PRECISION[Precisão]
    SELECT --> DISTANCE[Distância]
    SELECT --> SURVIVAL[Sobrevivência]
    SELECT --> DAILY[Desafio Diário]
    SELECT --> MP[Multiplayer Local]

    ARCADE --> LOAD[Carregar Mapa + Alvos]
    PRECISION --> LOAD
    DISTANCE --> LOAD
    SURVIVAL --> LOAD
    DAILY --> PROC[Procedural Gen]
    PROC --> LOAD
    MP --> LOAD

    LOAD --> PLAY[Loop de Jogo]
    PLAY --> END_CHECK{Fim da rodada?}
    END_CHECK -->|Não| PLAY
    END_CHECK -->|Sim| SCORE[Calcular Pontuação]
    SCORE --> XP[Atribuir XP]
    XP --> UNLOCK{Novos desbloqueios?}
    UNLOCK -->|Sim| NOTIFY[Notificar jogador]
    UNLOCK -->|Não| MENU
    NOTIFY --> MENU
```

## Loop de Frame (60 FPS)

```mermaid
flowchart LR
    subgraph Input
        I1[Mouse/Teclado]
        I2[Throw Charge]
    end

    subgraph Simulation
        S1[Weather Update]
        S2[Physics Integrate]
        S3[Collision Resolve]
        S4[AI Update]
        S5[Scoring Events]
    end

    subgraph Presentation
        P1[Sync Drawables]
        P2[Particles]
        P3[Render Scene]
        P4[Draw HUD]
        P5[Audio Triggers]
    end

    I1 --> I2
    I2 --> S1
    S1 --> S2
    S2 --> S3
    S3 --> S4
    S4 --> S5
    S5 --> P1
    P1 --> P2
    P2 --> P3
    P3 --> P4
    P4 --> P5
```

## Máquina de Estados — Arremesso

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Aiming: LMB down
    Aiming --> Aiming: adjust angle/spin
    Aiming --> Charging: hold LMB
    Charging --> Charging: force increases
    Charging --> Released: LMB up
    Released --> Flying: spawn rock
    Flying --> Flying: physics tick
    Flying --> Impact: collision
    Flying --> Grounded: stop moving
    Impact --> Scoring: target hit?
    Scoring --> Idle: reset
    Grounded --> Idle: reset
```

## Máquina de Estados — IA de Alvo

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Patrol: timer
    Patrol --> Alert: near miss / sound
    Alert --> Evasion: rock incoming
    Evasion --> SeekCover: threat persists
    SeekCover --> Patrol: safe
    Alert --> Patrol: timeout
    Evasion --> Patrol: escaped
```

## Pipeline de Física da Pedra

```mermaid
flowchart TD
    A[Estado: pos, vel, ω] --> B[Forças]
    B --> B1[Gravidade: m×g]
    B --> B2[Arrasto: ½ρCdAv²]
    B --> B3[Magnus: S×ω×v]
    B --> B4[Vento: exposição × v_wind]
    B1 & B2 & B3 & B4 --> C[Integração semi-implícita]
    C --> D{Colisão?}
    D -->|Terreno| E[Reflexão + atrito]
    D -->|Alvo| F[Transferência energia]
    D -->|Obstáculo| G[Ricochete]
    D -->|Nenhuma| H[Atualizar posição]
    E --> H
    F --> I[Evento de dano]
    G --> H
    H --> A
```

## Fluxo de Progressão

```mermaid
flowchart LR
    HIT[Acerto / Chefe] --> XP[+XP]
    XP --> LEVEL{Level up?}
    LEVEL -->|Sim| POINTS[+Skill Points]
    LEVEL -->|Não| PROFILE[Salvar profile]
    POINTS --> TREE[Skill Tree]
    LEVEL --> UNLOCK[Check Unlocks]
    UNLOCK --> NEW[Pedra / Mapa / Skin]
    TREE --> PROFILE
    NEW --> PROFILE
```
