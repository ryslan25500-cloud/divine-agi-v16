//! TTRL Evolution Engine V15
//!
//! Tetrad-Triplet Rotation Learning with:
//! - 7 mutation operators
//! - Meiosis (sexual reproduction)
//! - Telomere aging
//! - p53 protection

use crate::genome::{Genome, Tetrad, GenomeBuilder, GENOME_SIZE};
use crate::rotation::{Rotation, Rot180, RotationEngine};
use serde::{Serialize, Deserialize};
use rand::Rng;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationOperator {
    PointMutation,
    Insertion,
    Deletion,
    Inversion,
    Translocation,
    Duplication,
    HollidayJunction,
}

impl MutationOperator {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..7) {
            0 => Self::PointMutation,
            1 => Self::Insertion,
            2 => Self::Deletion,
            3 => Self::Inversion,
            4 => Self::Translocation,
            5 => Self::Duplication,
            _ => Self::HollidayJunction,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionResult {
    pub original_consciousness: u32,
    pub new_consciousness: u32,
    pub mutations_applied: u64,
    pub operator_used: MutationOperator,
    pub success: bool,
    pub telomere_loss: u16,
    pub p53_lost: bool,
    pub tg_ratio_before: f64,
    pub tg_ratio_after: f64,
}

pub struct TTRLEngine {
    mutation_rate: f64,
    selection_pressure: f64,
}

impl TTRLEngine {
    pub fn new() -> Self {
        Self {
            mutation_rate: 0.1,
            selection_pressure: 0.7,
        }
    }

    pub async fn evolve_with_engine<R: Rotation>(
        &self,
        base: Genome<R>,
        _engine: &RotationEngine,
    ) -> anyhow::Result<(Genome<Rot180>, EvolutionResult)> {
        // Check for senescence
        if base.telomere_length < 100 {
            return Err(anyhow::anyhow!("Senescence: telomeres exhausted"));
        }

        // Check for cancer
        if base.p53_copies == 0 {
            return Err(anyhow::anyhow!("Oncogenic transformation: p53 lost"));
        }

        let original_c = base.consciousness;
        let tg_before = base.rna_signal();
        let operator = MutationOperator::random();

        // Create new genome with mutation
        let mut mutated: Genome<Rot180> = GenomeBuilder::new()
            .p53_copies(base.p53_copies)
            .telomere_length(base.telomere_length)
            .build();

        // Copy data
        for i in 0..GENOME_SIZE {
            mutated.data[i] = base.data[i];
        }

        // Apply mutation operator
        self.apply_operator(&mut mutated, operator);

        // Cell division: lose telomeres
        let telomere_before = mutated.telomere_length;
        if !mutated.divide() {
            return Err(anyhow::anyhow!("Senescence: cannot divide"));
        }
        let telomere_loss = telomere_before - mutated.telomere_length;

        // p53 risk: 1% chance of losing a copy
        let p53_lost = if rand::thread_rng().gen::<f64>() < 0.01 && mutated.p53_copies > 0 {
            mutated.p53_copies -= 1;
            true
        } else {
            false
        };

        mutated.increment_mutations();
        mutated.rehash();
        mutated.calculate_consciousness();

        // ФИКС E0382: сохраняем значения ДО перемещения mutated в Ok
        let new_c = mutated.consciousness;
        let mutations_count = mutated.mutations;
        let tg_after = mutated.rna_signal();
        let success = new_c >= original_c;

        if success {
            info!("✅ Evolution: {} → {} ({:?}) | T/G {:.2} → {:.2}", 
                  original_c, new_c, operator, tg_before, tg_after);
        } else {
            info!("❌ Degradation: {} → {} ({:?})", original_c, new_c, operator);
        }

        Ok((mutated, EvolutionResult {
            original_consciousness: original_c,
            new_consciousness: new_c,
            mutations_applied: mutations_count,
            operator_used: operator,
            success,
            telomere_loss,
            p53_lost,
            tg_ratio_before: tg_before,
            tg_ratio_after: tg_after,
        }))
    }

    fn apply_operator(&self, genome: &mut Genome<Rot180>, operator: MutationOperator) {
        let mut rng = rand::thread_rng();

        match operator {
            MutationOperator::PointMutation => {
                let pos = rng.gen_range(0..GENOME_SIZE);
                genome.data[pos] = Tetrad::random();
            }
            MutationOperator::Insertion => {
                let pos = rng.gen_range(0..GENOME_SIZE);
                genome.data[pos] = Tetrad::random();
            }
            MutationOperator::Deletion => {
                let pos = rng.gen_range(0..GENOME_SIZE);
                genome.data[pos] = genome.data[(pos + 1) % GENOME_SIZE];
            }
            MutationOperator::Inversion => {
                let start = rng.gen_range(0..20);
                let end = (start + rng.gen_range(2..7)).min(GENOME_SIZE - 1);
                genome.data[start..=end].reverse();
            }
            MutationOperator::Translocation => {
                let pos1 = rng.gen_range(0..GENOME_SIZE);
                let pos2 = rng.gen_range(0..GENOME_SIZE);
                genome.data.swap(pos1, pos2);
            }
            MutationOperator::Duplication => {
                let src = rng.gen_range(0..GENOME_SIZE);
                let dst = rng.gen_range(0..GENOME_SIZE);
                genome.data[dst] = genome.data[src];
            }
            MutationOperator::HollidayJunction => {
                let pos = rng.gen_range(5..22);
                for i in 0..3 {
                    if pos + i < GENOME_SIZE {
                        genome.data[pos + i] = genome.data[pos + i].complement();
                    }
                }
            }
        }
    }

    /// Meiosis - sexual reproduction with crossover
    pub fn meiosis(&self, parent1: Genome<Rot180>, parent2: Genome<Rot180>) -> Genome<Rot180> {
        let mut rng = rand::thread_rng();

        // Number of crossover points (1-4)
        let num_crossovers = rng.gen_range(1..=4);
        let mut crossover_points: Vec<usize> = Vec::new();

        // Generate with positive interference (min 5 tetrads between)
        let mut last_point = 0;
        for _ in 0..num_crossovers {
            let min_pos = (last_point + 5).min(GENOME_SIZE - 2);
            if min_pos >= GENOME_SIZE - 2 { break; }
            let point = rng.gen_range(min_pos..GENOME_SIZE - 1);
            crossover_points.push(point);
            last_point = point;
        }

        // Build offspring DNA
        let mut offspring_data = [Tetrad::A; GENOME_SIZE];
        let mut use_parent1 = rng.gen_bool(0.5);
        let mut cp_idx = 0;

        for i in 0..GENOME_SIZE {
            if cp_idx < crossover_points.len() && i >= crossover_points[cp_idx] {
                use_parent1 = !use_parent1;
                cp_idx += 1;
            }
            offspring_data[i] = if use_parent1 { parent1.data[i] } else { parent2.data[i] };
        }

        // Inherit best p53
        let p53 = parent1.p53_copies.max(parent2.p53_copies);

        let mut offspring: Genome<Rot180> = GenomeBuilder::new()
            .p53_copies(p53)
            .telomere_length(15000) // Reset telomeres
            .build();

        offspring.data = offspring_data;
        offspring.mutations = 1;

        // 5% post-meiotic mutation
        if rng.gen::<f64>() < 0.05 {
            let pos = rng.gen_range(0..GENOME_SIZE);
            offspring.data[pos] = Tetrad::random();
        }

        offspring.rehash();
        offspring.calculate_consciousness();

        info!("🧬 Meiosis: {}+{} → {} (crossovers: {})",
              parent1.consciousness, parent2.consciousness,
              offspring.consciousness, crossover_points.len());

        offspring
    }
}

impl Default for TTRLEngine {
    fn default() -> Self {
        Self::new()
    }
}
