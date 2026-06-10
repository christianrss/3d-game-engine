//! Árvore de habilidades.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillBranch {
    Strength,
    Precision,
    Technique,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillNode {
    StrongArm,
    DoubleThrow,
    Burst,
    EagleEye,
    TrajectoryVision,
    SlowAim,
    CalmWind,
    ControlledBounce,
    SpinMaster,
}

impl SkillNode {
    pub fn branch(self) -> SkillBranch {
        match self {
            SkillNode::StrongArm | SkillNode::DoubleThrow | SkillNode::Burst => SkillBranch::Strength,
            SkillNode::EagleEye | SkillNode::TrajectoryVision | SkillNode::SlowAim => SkillBranch::Precision,
            SkillNode::CalmWind | SkillNode::ControlledBounce | SkillNode::SpinMaster => SkillBranch::Technique,
        }
    }

    pub fn cost(self) -> u32 {
        match self {
            SkillNode::StrongArm | SkillNode::EagleEye | SkillNode::CalmWind => 1,
            SkillNode::DoubleThrow | SkillNode::TrajectoryVision | SkillNode::ControlledBounce => 2,
            SkillNode::Burst | SkillNode::SlowAim | SkillNode::SpinMaster => 3,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            SkillNode::StrongArm => "Braço Forte",
            SkillNode::DoubleThrow => "Arremesso Duplo",
            SkillNode::Burst => "Rajada",
            SkillNode::EagleEye => "Olho de Águia",
            SkillNode::TrajectoryVision => "Visão de Trajetória",
            SkillNode::SlowAim => "Mira Lenta",
            SkillNode::CalmWind => "Vento Calmo",
            SkillNode::ControlledBounce => "Ricochete Controlado",
            SkillNode::SpinMaster => "Spin Mestre",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillTree {
    pub unlocked: Vec<SkillNode>,
}

impl SkillTree {
    pub fn unlock(&mut self, node: SkillNode, points: &mut u32) -> bool {
        if self.unlocked.contains(&node) {
            return false;
        }
        let cost = node.cost();
        if *points < cost {
            return false;
        }
        *points -= cost;
        self.unlocked.push(node);
        true
    }

    pub fn has(&self, node: SkillNode) -> bool {
        self.unlocked.contains(&node)
    }

    pub fn force_bonus(&self) -> f32 {
        if self.has(SkillNode::StrongArm) { 0.10 } else { 0.0 }
    }

    pub fn dispersion_reduction(&self) -> f32 {
        if self.has(SkillNode::EagleEye) { 0.20 } else { 0.0 }
    }

    pub fn wind_reduction(&self) -> f32 {
        if self.has(SkillNode::CalmWind) { 0.30 } else { 0.0 }
    }

    pub fn trajectory_preview(&self) -> bool {
        self.has(SkillNode::TrajectoryVision)
    }
}
