//! FSM modular para NPCs e alvos inteligentes.

use crate::games::rock_3d::targets::TargetInstance;
use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiState {
    Idle,
    Patrol,
    Alert,
    Evasion,
    SeekCover,
}

#[derive(Debug, Clone)]
pub struct AiAgent {
    pub state: AiState,
    pub patrol_origin: Vec3,
    pub patrol_radius: f32,
    pub alert_timer: f32,
    pub cover_position: Option<Vec3>,
    pub speed: f32,
}

impl AiAgent {
    pub fn new(origin: Vec3) -> Self {
        Self {
            state: AiState::Patrol,
            patrol_origin: origin,
            patrol_radius: 5.0,
            alert_timer: 0.0,
            cover_position: None,
            speed: 2.5,
        }
    }
}

pub struct AiSystem;

impl AiSystem {
    pub fn update(agent: &mut AiAgent, target: &mut TargetInstance, dt: f32, threat_pos: Option<Vec3>) {
        match agent.state {
            AiState::Idle => {
                agent.alert_timer += dt;
                if agent.alert_timer > 3.0 {
                    agent.state = AiState::Patrol;
                    agent.alert_timer = 0.0;
                }
            }
            AiState::Patrol => {
                let t = agent.alert_timer + dt;
                agent.alert_timer = t;
                let offset = Vec3::new(t.sin() * agent.patrol_radius, 0.0, t.cos() * agent.patrol_radius * 0.5);
                target.position = agent.patrol_origin + offset;
                target.position.y = agent.patrol_origin.y;

                if threat_pos.is_some() {
                    agent.state = AiState::Alert;
                    agent.alert_timer = 0.0;
                }
            }
            AiState::Alert => {
                agent.alert_timer += dt;
                if let Some(threat) = threat_pos {
                    let away = (target.position - threat).normalize_or_zero();
                    target.velocity = away * agent.speed;
                    target.position += target.velocity * dt;
                    if (target.position - threat).length() > 15.0 {
                        agent.state = AiState::Patrol;
                    } else if agent.alert_timer > 1.5 {
                        agent.state = AiState::Evasion;
                    }
                } else if agent.alert_timer > 5.0 {
                    agent.state = AiState::Patrol;
                }
            }
            AiState::Evasion => {
                if let Some(threat) = threat_pos {
                    let away = (target.position - threat).normalize_or_zero();
                    target.velocity = away * agent.speed * 2.0;
                    target.position += target.velocity * dt;
                    if (target.position - threat).length() > 20.0 {
                        agent.state = AiState::SeekCover;
                        agent.cover_position = Some(target.position + away * 5.0);
                    }
                }
            }
            AiState::SeekCover => {
                if let Some(cover) = agent.cover_position {
                    let to_cover = cover - target.position;
                    if to_cover.length() > 0.5 {
                        target.velocity = to_cover.normalize() * agent.speed;
                        target.position += target.velocity * dt;
                    } else {
                        agent.state = AiState::Patrol;
                        agent.alert_timer = 0.0;
                    }
                } else {
                    agent.state = AiState::Patrol;
                }
            }
        }
    }

    pub fn on_near_miss(agent: &mut AiAgent, _target: &TargetInstance) {
        agent.state = AiState::Alert;
        agent.alert_timer = 0.0;
    }
}
