//! Rotation System V15 — Divine Kernel v3
//!
//! Dynamic 4-state rotation:
//! - Rot0   (0°)   - Active/Compute (эухроматин)
//! - Rot90  (90°)  - Processing/Balance
//! - Rot180 (180°) - Storage (гетерохроматин, БД)
//! - Rot270 (270°) - Mutation (TTRL эволюция)

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Trait for rotation states
pub trait Rotation: Clone + Send + Sync + 'static {
    const ANGLE: u16;
    const NAME: &'static str;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rot0;
impl Rotation for Rot0 {
    const ANGLE: u16 = 0;
    const NAME: &'static str = "Active";
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rot90;
impl Rotation for Rot90 {
    const ANGLE: u16 = 90;
    const NAME: &'static str = "Processing";
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rot180;
impl Rotation for Rot180 {
    const ANGLE: u16 = 180;
    const NAME: &'static str = "Storage";
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rot270;
impl Rotation for Rot270 {
    const ANGLE: u16 = 270;
    const NAME: &'static str = "Mutation";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DynamicRotation {
    Rot0,
    Rot90,
    Rot180,
    Rot270,
}

impl DynamicRotation {
    pub fn angle(&self) -> u16 {
        match self {
            Self::Rot0 => 0,
            Self::Rot90 => 90,
            Self::Rot180 => 180,
            Self::Rot270 => 270,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Rot0 => "Active",
            Self::Rot90 => "Processing",
            Self::Rot180 => "Storage",
            Self::Rot270 => "Mutation",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Rot0 => Self::Rot90,
            Self::Rot90 => Self::Rot180,
            Self::Rot180 => Self::Rot270,
            Self::Rot270 => Self::Rot0,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Self::Rot0 => Self::Rot270,
            Self::Rot90 => Self::Rot0,
            Self::Rot180 => Self::Rot90,
            Self::Rot270 => Self::Rot180,
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Rot0 => "⚡",
            Self::Rot90 => "⚖️",
            Self::Rot180 => "💾",
            Self::Rot270 => "🧬",
        }
    }
}

impl Default for DynamicRotation {
    fn default() -> Self {
        Self::Rot180 // Storage default
    }
}

impl std::fmt::Display for DynamicRotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}° ({})", self.angle(), self.name())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationStats {
    pub current_rotation: DynamicRotation,
    pub total_rotations: u64,
    pub rotations_per_state: HashMap<String, u64>,
    pub active_genomes: u64,
    pub last_rotation_time: i64,
}

impl RotationStats {
    pub fn from_engine(engine: &RotationEngine) -> Self {
        let mut per_state = HashMap::new();
        per_state.insert("Rot0".into(), engine.rot0_count);
        per_state.insert("Rot90".into(), engine.rot90_count);
        per_state.insert("Rot180".into(), engine.rot180_count);
        per_state.insert("Rot270".into(), engine.rot270_count);

        Self {
            current_rotation: engine.current,
            total_rotations: engine.total_rotations,
            rotations_per_state: per_state,
            active_genomes: engine.active_genomes,
            last_rotation_time: engine.last_rotation_time,
        }
    }
}

#[derive(Debug)]
pub struct RotationEngine {
    pub current: DynamicRotation,
    pub total_rotations: u64,
    pub rot0_count: u64,
    pub rot90_count: u64,
    pub rot180_count: u64,
    pub rot270_count: u64,
    pub active_genomes: u64,
    pub last_rotation_time: i64,
}

impl RotationEngine {
    pub fn new() -> Self {
        Self {
            current: DynamicRotation::Rot180,
            total_rotations: 0,
            rot0_count: 0,
            rot90_count: 0,
            rot180_count: 0,
            rot270_count: 0,
            active_genomes: 0,
            last_rotation_time: chrono::Utc::now().timestamp(),
        }
    }

    pub fn rotate(&mut self) -> DynamicRotation {
        self.current = self.current.next();
        self.total_rotations += 1;
        self.last_rotation_time = chrono::Utc::now().timestamp();

        match self.current {
            DynamicRotation::Rot0 => self.rot0_count += 1,
            DynamicRotation::Rot90 => self.rot90_count += 1,
            DynamicRotation::Rot180 => self.rot180_count += 1,
            DynamicRotation::Rot270 => self.rot270_count += 1,
        }

        self.current
    }

    pub fn rotate_to(&mut self, target: DynamicRotation) {
        while self.current != target {
            self.rotate();
        }
    }

    pub fn current(&self) -> DynamicRotation {
        self.current
    }

    pub fn increment_active(&mut self) {
        self.active_genomes += 1;
    }

    pub fn decrement_active(&mut self) {
        self.active_genomes = self.active_genomes.saturating_sub(1);
    }

    pub fn get_stats(&self) -> RotationStats {
        RotationStats::from_engine(self)
    }
}

impl Default for RotationEngine {
    fn default() -> Self {
        Self::new()
    }
}
