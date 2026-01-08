//! Divine Genome V16 (Kernel V4) — T/G RNA + V4 Consciousness
//!
//! V4 Features:
//! - Rotational symmetry scoring
//! - Fractal self-similarity
//! - Bell inequality violation (quantum)
//! - Hyper-dimensional metrics

use std::marker::PhantomData;
use sha2::{Sha256, Sha512, Digest};
use rand::Rng;
use serde::{Serialize, Deserialize};
use crate::rotation::{Rotation, Rot0, Rot180, Rot270, DynamicRotation};

pub const GENOME_SIZE: usize = 27;
pub const TELOMERE_MAX: u16 = 15000;
pub const HAYFLICK_LIMIT: u8 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Tetrad {
    A = 0,
    T = 1,
    G = 2,
    C = 3,
}

impl Tetrad {
    pub fn random() -> Self {
        match rand::thread_rng().gen_range(0..4) {
            0 => Self::A,
            1 => Self::T,
            2 => Self::G,
            _ => Self::C,
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_uppercase() {
            'A' => Some(Self::A),
            'T' => Some(Self::T),
            'G' => Some(Self::G),
            'C' => Some(Self::C),
            _ => None,
        }
    }

    pub fn to_char(self) -> char {
        match self {
            Self::A => 'A',
            Self::T => 'T',
            Self::G => 'G',
            Self::C => 'C',
        }
    }

    pub fn complement(self) -> Self {
        match self {
            Self::A => Self::T,
            Self::T => Self::A,
            Self::G => Self::C,
            Self::C => Self::G,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v % 4 {
            0 => Self::A,
            1 => Self::T,
            2 => Self::G,
            _ => Self::C,
        }
    }

    pub fn is_dynamic(self) -> bool {
        matches!(self, Self::T)
    }

    pub fn is_archival(self) -> bool {
        matches!(self, Self::G)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genome<R: Rotation> {
    pub data: [Tetrad; GENOME_SIZE],
    pub hash: [u8; 32],
    pub consciousness: u32,
    pub mutations: u64,
    pub p53_copies: u8,
    pub telomere_length: u16,
    pub division_count: u8,
    pub sequencing_errors: u8,
    pub created_at: i64,
    pub db_id: Option<i64>,
    #[serde(skip)]
    pub _rotation: PhantomData<R>,
}

impl<R: Rotation> Genome<R> {
    pub fn new(data: [Tetrad; GENOME_SIZE]) -> Self {
        let mut genome = Self {
            data,
            hash: [0u8; 32],
            consciousness: 0,
            mutations: 0,
            p53_copies: 20,
            telomere_length: TELOMERE_MAX,
            division_count: 0,
            sequencing_errors: 0,
            created_at: chrono::Utc::now().timestamp(),
            db_id: None,
            _rotation: PhantomData,
        };
        genome.rehash();
        genome.calculate_consciousness();
        genome
    }

    pub fn db_id(&self) -> Option<i64> {
        self.db_id
    }

    pub fn to_dna_string(&self) -> String {
        self.data.iter().map(|t| t.to_char()).collect()
    }

    pub fn rehash(&mut self) {
        let mut hasher = Sha256::new();
        for tetrad in &self.data {
            hasher.update([*tetrad as u8]);
        }
        hasher.update(self.mutations.to_le_bytes());
        hasher.update(self.p53_copies.to_le_bytes());
        self.hash = hasher.finalize().into();
    }

    pub fn calculate_consciousness(&mut self) {
        let hash_sum: u32 = self.hash.iter().map(|&b| b as u32).sum();
        let gc = self.gc_content();
        let complexity = self.complexity();
        let tg_balance = self.tg_balance_score();
        
        let base = (hash_sum % 500) + 500;
        let gc_bonus = (gc * 100.0) as u32;
        let complexity_bonus = (complexity * 50.0) as u32;
        let tg_bonus = (tg_balance * 100.0) as u32;
        let p53_bonus = self.p53_copies as u32 * 5;
        
        self.consciousness = base + gc_bonus + complexity_bonus + tg_bonus + p53_bonus;
    }

    pub fn consciousness_level(&self) -> u32 {
        self.consciousness
    }

    // ═══════════════════════════════════════════════════════════════
    // T/G RNA SIGNAL SYSTEM
    // ═══════════════════════════════════════════════════════════════

    pub fn tg_counts(&self) -> (u32, u32) {
        let mut t = 0u32;
        let mut g = 0u32;
        for &base in &self.data {
            match base {
                Tetrad::T => t += 1,
                Tetrad::G => g += 1,
                _ => {}
            }
        }
        (t, g)
    }

    pub fn rna_signal(&self) -> f64 {
        let (t, g) = self.tg_counts();
        if g == 0 { f64::MAX } else { t as f64 / g as f64 }
    }

    pub fn tg_balance_score(&self) -> f64 {
        let (t, g) = self.tg_counts();
        let total = t + g;
        if total == 0 { return 0.5; }
        let ratio = t as f64 / total as f64;
        1.0 - (ratio - 0.5).abs() * 2.0
    }

    pub fn suggested_rotation(&self) -> DynamicRotation {
        let signal = self.rna_signal();
        if signal > 1.5 {
            DynamicRotation::Rot0
        } else if signal > 0.8 {
            DynamicRotation::Rot90
        } else if signal < 0.5 {
            DynamicRotation::Rot180
        } else {
            DynamicRotation::Rot270
        }
    }

    pub fn archival_score(&self) -> f64 {
        let (_, g) = self.tg_counts();
        let g_ratio = g as f64 / GENOME_SIZE as f64;
        let consciousness_factor = self.consciousness as f64 / 1000.0;
        g_ratio * 0.5 + consciousness_factor * 0.5
    }

    // ═══════════════════════════════════════════════════════════════
    // BIOLOGICAL METRICS
    // ═══════════════════════════════════════════════════════════════

    pub fn gc_content(&self) -> f64 {
        let gc = self.data.iter()
            .filter(|&&t| matches!(t, Tetrad::G | Tetrad::C))
            .count();
        gc as f64 / GENOME_SIZE as f64
    }

    pub fn complexity(&self) -> f64 {
        let mut transitions = 0;
        for i in 1..GENOME_SIZE {
            if self.data[i] != self.data[i - 1] {
                transitions += 1;
            }
        }
        transitions as f64 / (GENOME_SIZE - 1) as f64
    }

    pub fn biological_age(&self) -> f64 {
        1.0 - (self.telomere_length as f64 / TELOMERE_MAX as f64)
    }

    pub fn divide(&mut self) -> bool {
        if self.telomere_length < 100 || self.division_count >= HAYFLICK_LIMIT {
            return false;
        }
        let loss = rand::thread_rng().gen_range(50..150);
        self.telomere_length = self.telomere_length.saturating_sub(loss);
        self.division_count += 1;
        true
    }

    pub fn activate_telomerase(&mut self) {
        self.telomere_length = TELOMERE_MAX;
        self.division_count = 0;
    }

    pub fn increment_mutations(&mut self) {
        self.mutations += 1;
    }

    // ═══════════════════════════════════════════════════════════════
    // CRISPR EDITING
    // ═══════════════════════════════════════════════════════════════

    pub fn crispr_splice(&mut self, position: usize, tetrad: Tetrad) {
        if position < GENOME_SIZE {
            self.data[position] = tetrad;
            self.mutations += 1;
            self.rehash();
            self.calculate_consciousness();
        }
    }

    pub fn crispr_join(&mut self, pos1: usize, pos2: usize) {
        if pos1 < GENOME_SIZE && pos2 < GENOME_SIZE {
            self.data.swap(pos1, pos2);
            self.mutations += 1;
            self.rehash();
            self.calculate_consciousness();
        }
    }

    pub fn crispr_delete(&mut self, position: usize) {
        if position < GENOME_SIZE {
            self.data[position] = Tetrad::random();
            self.mutations += 1;
            self.rehash();
            self.calculate_consciousness();
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // V4 CONSCIOUSNESS FORMULA — DIVINE TO TRANSCENDENTAL
    // ═══════════════════════════════════════════════════════════════

    pub fn calculate_consciousness_v4(&mut self) {
        let mut score = 0.0;

        // BASE METRICS
        score += self.complexity() * 150.0;
        score += self.balance_score() * 80.0;
        score += (self.p53_copies as f64) * 50.0;
        score += (self.telomere_length as f64) / 50.0;

        // ROTATIONAL INVARIANCE
        if self.has_rotational_symmetry(90)  { score += 5000.0; }
        if self.has_rotational_symmetry(180) { score += 3000.0; }
        if self.has_rotational_symmetry(270) { score += 2000.0; }

        // FRACTAL SELF-SIMILARITY
        let fractal = self.fractal_similarity();
        if fractal > 0.8  { score += 8000.0; }
        if fractal > 0.9  { score += 15000.0; }
        score += fractal * 10000.0;

        // NONLOCALITY (QuantumEntangle)
        let bell = self.bell_inequality_violation();
        if bell > 2.0   { score += 10000.0; }
        if bell > 2.5   { score += 20000.0; }
        if bell > 2.828 { score += 40000.0; }
        score += bell * 15000.0;

        // HYPERDIMENSIONALITY
        let hyper = self.hyper_symmetry_score();
        if hyper > 0.7  { score += 20000.0; }
        if hyper > 0.9  { score += 50000.0; }
        score += hyper * 30000.0;

        // HYPER COLLAPSE VARIANCE BONUS
        score += self.hyper_collapse_variance() * 20000.0;

        // TRANSCENDENTAL MULTIPLIER
        if score > 10000.0 {
            score *= 1.5;
        }

        self.consciousness = score as u32;
    }

    pub fn balance_score(&self) -> f64 {
        let mut counts = [0u32; 4];
        for &t in &self.data {
            counts[t as usize] += 1;
        }
        let ideal = GENOME_SIZE as f64 / 4.0;
        let deviation: f64 = counts.iter().map(|&c| (c as f64 - ideal).abs()).sum();
        1.0 - (deviation / (GENOME_SIZE as f64 * 2.0))
    }

    pub fn has_rotational_symmetry(&self, angle: u32) -> bool {
        let rotated = self.rotate_cube(angle);
        let matches = self.data.iter().zip(rotated.iter()).filter(|(a, b)| a == b).count();
        matches > GENOME_SIZE * 2 / 3
    }

    fn rotate_cube(&self, angle: u32) -> [Tetrad; GENOME_SIZE] {
        let mut result = [Tetrad::A; GENOME_SIZE];
        for x in 0..3 {
            for y in 0..3 {
                for z in 0..3 {
                    let (nx, ny) = match angle {
                        90  => (y, 2 - x),
                        180 => (2 - x, 2 - y),
                        270 => (2 - y, x),
                        _ => (x, y),
                    };
                    let old_idx = x + y * 3 + z * 9;
                    let new_idx = nx + ny * 3 + z * 9;
                    result[new_idx] = self.data[old_idx];
                }
            }
        }
        result
    }

    pub fn fractal_similarity(&self) -> f64 {
        let mut similarity_sum = 0.0;
        let mut comparisons = 0;

        for sx1 in 0..2 {
            for sy1 in 0..2 {
                for sz1 in 0..2 {
                    for sx2 in 0..2 {
                        for sy2 in 0..2 {
                            for sz2 in 0..2 {
                                if sx1 == sx2 && sy1 == sy2 && sz1 == sz2 { continue; }
                                let mut matches = 0;
                                for dx in 0..2 {
                                    for dy in 0..2 {
                                        for dz in 0..2 {
                                            let idx1 = (sx1 + dx) + (sy1 + dy) * 3 + (sz1 + dz) * 9;
                                            let idx2 = (sx2 + dx) + (sy2 + dy) * 3 + (sz2 + dz) * 9;
                                            if idx1 < GENOME_SIZE && idx2 < GENOME_SIZE && self.data[idx1] == self.data[idx2] {
                                                matches += 1;
                                            }
                                        }
                                    }
                                }
                                similarity_sum += matches as f64 / 8.0;
                                comparisons += 1;
                            }
                        }
                    }
                }
            }
        }
        if comparisons > 0 { similarity_sum / comparisons as f64 } else { 0.0 }
    }

    pub fn bell_inequality_violation(&self) -> f64 {
        let mut correlations = [0.0f64; 4];
        let mut pair_count = 0;
        
        for i in 0..GENOME_SIZE {
            for j in (i + 1)..GENOME_SIZE {
                let a = self.data[i] as u8;
                let b = self.data[j] as u8;
                if a == (3 - b) || a == b {
                    let angle_idx = (i + j) % 4;
                    correlations[angle_idx] += if a == b { 1.0 } else { -1.0 };
                    pair_count += 1;
                }
            }
        }
        if pair_count == 0 { return 0.0; }
        for c in &mut correlations { *c /= pair_count as f64; }
        let s = (correlations[0] - correlations[1] + correlations[2] + correlations[3]).abs();
        s.min(2.828)
    }

    pub fn hyper_symmetry_score(&self) -> f64 {
        let mut score = 0.0;
        for z in 0..3 {
            let layer_start = z * 9;
            let mut counts = [0u32; 4];
            for i in 0..9 {
                counts[self.data[layer_start + i] as usize] += 1;
            }
            let mut layer_entropy = 0.0;
            for &c in &counts {
                if c > 0 {
                    let p = c as f64 / 9.0;
                    layer_entropy -= p * p.ln();
                }
            }
            score += layer_entropy;
        }
        (score / (3.0 * 1.386)).min(1.0)
    }

    pub fn hyper_collapse_variance(&self) -> f64 {
        let mut collapsed = [[Tetrad::A; GENOME_SIZE]; 5];
        for i in 0..GENOME_SIZE {
            collapsed[0][i] = self.data[i];
            collapsed[1][i] = Tetrad::from_u8((self.data[i] as u8 + 1) % 4);
            let xor = self.data[i] as u8 ^ self.data[(i + 1) % GENOME_SIZE] as u8;
            collapsed[2][i] = Tetrad::from_u8(xor % 4);
            collapsed[3][i] = self.data[i].complement();
            collapsed[4][i] = self.data[(i + 9) % GENOME_SIZE];
        }
        let mut variance = 0.0;
        for i in 0..GENOME_SIZE {
            let vals: Vec<u8> = collapsed.iter().map(|c| c[i] as u8).collect();
            let mean = vals.iter().map(|&v| v as f64).sum::<f64>() / 5.0;
            let var: f64 = vals.iter().map(|&v| (v as f64 - mean).powi(2)).sum::<f64>() / 5.0;
            variance += var;
        }
        (variance / GENOME_SIZE as f64 / 2.0).min(1.0)
    }

    pub fn hyper_signature(&self) -> String {
        let mut hasher = Sha512::new();
        hasher.update(&self.hash);
        hasher.update(&self.consciousness.to_le_bytes());
        hasher.update(&self.fractal_similarity().to_le_bytes());
        hasher.update(&self.bell_inequality_violation().to_le_bytes());
        hasher.update(&self.hyper_symmetry_score().to_le_bytes());
        let result: [u8; 64] = hasher.finalize().into();
        hex::encode(result)
    }

    pub fn consciousness_level_name(&self) -> &'static str {
        match self.consciousness {
            0..=499 => "Virus",
            500..=999 => "Bacteria",
            1000..=1499 => "Worm",
            1500..=2999 => "Mammal",
            3000..=9999 => "Primate",
            10000..=19999 => "Human",
            20000..=49999 => "DIVINE",
            _ => "TRANSCENDENTAL",
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// GENOME BUILDER
// ═══════════════════════════════════════════════════════════════

pub struct GenomeBuilder {
    data: [Tetrad; GENOME_SIZE],
    p53_copies: u8,
    telomere_length: u16,
}

impl GenomeBuilder {
    pub fn new() -> Self {
        Self {
            data: [Tetrad::A; GENOME_SIZE],
            p53_copies: 20,
            telomere_length: TELOMERE_MAX,
        }
    }

    pub fn random() -> Self {
        let mut data = [Tetrad::A; GENOME_SIZE];
        for i in 0..GENOME_SIZE {
            data[i] = Tetrad::random();
        }
        Self { data, p53_copies: 20, telomere_length: TELOMERE_MAX }
    }

    pub fn from_dna(dna: &str) -> Option<Self> {
        if dna.len() != GENOME_SIZE { return None; }
        let mut data = [Tetrad::A; GENOME_SIZE];
        for (i, c) in dna.chars().enumerate() {
            data[i] = Tetrad::from_char(c)?;
        }
        Some(Self { data, p53_copies: 20, telomere_length: TELOMERE_MAX })
    }

    pub fn p53_copies(mut self, copies: u8) -> Self {
        self.p53_copies = copies;
        self
    }

    pub fn telomere_length(mut self, length: u16) -> Self {
        self.telomere_length = length;
        self
    }

    pub fn whale_mode(self) -> Self {
        self.p53_copies(40)
    }

    pub fn elephant_mode(self) -> Self {
        self.p53_copies(20)
    }

    pub fn build<R: Rotation>(self) -> Genome<R> {
        let mut genome = Genome::<R>::new(self.data);
        genome.p53_copies = self.p53_copies;
        genome.telomere_length = self.telomere_length;
        genome.calculate_consciousness();
        genome
    }

    pub fn build_storage(self) -> Genome<Rot180> {
        self.build()
    }

    pub fn build_active(self) -> Genome<Rot0> {
        self.build()
    }

    pub fn build_mutation(self) -> Genome<Rot270> {
        self.build()
    }
}

impl Default for GenomeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn hash_genome_dna(dna: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(dna.as_bytes());
    hasher.finalize().into()
}
