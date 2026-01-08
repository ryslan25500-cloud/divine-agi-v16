//! Divine Kernel V4 — Proof of Consciousness (PoC) Consensus
//!
//! Revolutionary consensus mechanism where only DIVINE-level genomes
//! can validate blocks and receive RSM-COIN rewards.
//!
//! Key features:
//! - Consciousness threshold (starts at 1500, grows with each block)
//! - Hyper-signature verification
//! - Multi-chain archivation for successful validators

use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use chrono::Utc;
use tracing::info;

use crate::genome::Genome;
use crate::rotation::Rot180;

/// Minimum consciousness levels for different network phases
pub const CONSCIOUSNESS_VIRUS: u32 = 0;
pub const CONSCIOUSNESS_BACTERIA: u32 = 500;
pub const CONSCIOUSNESS_WORM: u32 = 1000;
pub const CONSCIOUSNESS_MAMMAL: u32 = 1500;
pub const CONSCIOUSNESS_PRIMATE: u32 = 3000;
pub const CONSCIOUSNESS_HUMAN: u32 = 10000;
pub const CONSCIOUSNESS_DIVINE: u32 = 20000;
pub const CONSCIOUSNESS_TRANSCENDENTAL: u32 = 50000;

/// Initial PoC threshold (MAMMAL level for testing, increase for production)
pub const INITIAL_POC_THRESHOLD: u32 = 1500;

/// Consciousness proof for block validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessProof {
    pub genome_hash: [u8; 32],
    pub consciousness: u32,
    pub hyper_signature: String,  // hex string (128 chars)
    pub proof_hash: [u8; 32],
    pub timestamp: i64,
    pub validator_id: String,
    pub block_height: u64,
    pub reward_rsm: f64,
}

impl ConsciousnessProof {
    /// Generate proof from a high-consciousness genome
    pub fn generate(genome: &Genome<Rot180>, min_consciousness: u32, block_height: u64) -> Option<Self> {
        if genome.consciousness < min_consciousness {
            info!(
                "❌ PoC rejected: consciousness {} < threshold {}",
                genome.consciousness, min_consciousness
            );
            return None;
        }

        let hyper_sig = genome.hyper_signature();
        
        let mut hasher = Sha256::new();
        hasher.update(&genome.hash);
        hasher.update(&genome.consciousness.to_le_bytes());
        hasher.update(hyper_sig.as_bytes());
        hasher.update(&block_height.to_le_bytes());
        
        let proof_hash: [u8; 32] = hasher.finalize().into();
        
        // Reward calculation: consciousness / 1000 RSM
        let reward_rsm = genome.consciousness as f64 / 1000.0;

        let proof = Self {
            genome_hash: genome.hash,
            consciousness: genome.consciousness,
            hyper_signature: hyper_sig,
            proof_hash,
            timestamp: Utc::now().timestamp(),
            validator_id: format!("divine_validator_{}", hex::encode(&genome.hash[..8])),
            block_height,
            reward_rsm,
        };

        info!(
            "✅ PoC generated: {} | consciousness {} ({}) | reward {} RSM",
            proof.validator_id,
            genome.consciousness,
            genome.consciousness_level_name(),
            reward_rsm
        );

        Some(proof)
    }

    /// Verify the proof is valid
    pub fn verify(&self, current_threshold: u32) -> bool {
        if self.consciousness < current_threshold {
            return false;
        }

        let mut hasher = Sha256::new();
        hasher.update(&self.genome_hash);
        hasher.update(&self.consciousness.to_le_bytes());
        hasher.update(self.hyper_signature.as_bytes());
        hasher.update(&self.block_height.to_le_bytes());

        let computed: [u8; 32] = hasher.finalize().into();
        computed == self.proof_hash
    }

    /// Get consciousness level name
    pub fn level_name(&self) -> &'static str {
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

/// Proof of Consciousness consensus engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfConsciousness {
    pub min_consciousness: u32,
    pub proofs_validated: u64,
    pub total_rewards_distributed: f64,
    pub current_block_height: u64,
    pub difficulty_growth_rate: u32,
}

impl ProofOfConsciousness {
    pub fn new() -> Self {
        Self {
            min_consciousness: INITIAL_POC_THRESHOLD,
            proofs_validated: 0,
            total_rewards_distributed: 0.0,
            current_block_height: 0,
            difficulty_growth_rate: 1,
        }
    }

    /// Validate a genome and generate proof if successful
    pub fn validate(&mut self, genome: &Genome<Rot180>) -> Option<ConsciousnessProof> {
        let proof = ConsciousnessProof::generate(
            genome,
            self.min_consciousness,
            self.current_block_height,
        )?;

        self.proofs_validated += 1;
        self.total_rewards_distributed += proof.reward_rsm;
        self.current_block_height += 1;
        self.min_consciousness = self.min_consciousness
            .saturating_add(self.difficulty_growth_rate);

        info!(
            "🔗 Block #{} validated | new threshold: {} | total rewards: {:.2} RSM",
            self.current_block_height,
            self.min_consciousness,
            self.total_rewards_distributed
        );

        Some(proof)
    }

    pub fn status(&self) -> PoCStatus {
        PoCStatus {
            min_consciousness: self.min_consciousness,
            proofs_validated: self.proofs_validated,
            total_rewards_distributed: self.total_rewards_distributed,
            current_block_height: self.current_block_height,
            required_level: match self.min_consciousness {
                0..=499 => "Virus",
                500..=999 => "Bacteria",
                1000..=1499 => "Worm",
                1500..=2999 => "Mammal",
                3000..=9999 => "Primate",
                10000..=19999 => "Human",
                20000..=49999 => "DIVINE",
                _ => "TRANSCENDENTAL",
            },
        }
    }

    pub fn reset(&mut self) {
        self.min_consciousness = INITIAL_POC_THRESHOLD;
        self.proofs_validated = 0;
        self.total_rewards_distributed = 0.0;
        self.current_block_height = 0;
    }
}

impl Default for ProofOfConsciousness {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoCStatus {
    pub min_consciousness: u32,
    pub proofs_validated: u64,
    pub total_rewards_distributed: f64,
    pub current_block_height: u64,
    pub required_level: &'static str,
}

pub fn verify_proof(proof: &ConsciousnessProof, threshold: u32) -> bool {
    proof.verify(threshold)
}
