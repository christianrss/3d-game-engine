//! Rock 3D — jogo competitivo de arremesso de pedras.
//!
//! ```bash
//! cargo run --bin rock-3d
//! ```
//!
//! Controles:
//! - WASD: mover | Mouse: mirar
//! - Segurar LMB: carregar força | Soltar: arremessar
//! - Q/E: spin lateral | R/F: spin superior/inferior
//! - Setas: ajustar ângulo H/V
//! - 1-7: trocar pedra | N: nova rodada | Esc: pausar/sair

use desert_shooter_engine::games::rock_3d::Rock3DApp;

fn main() {
    Rock3DApp::new().run();
}
