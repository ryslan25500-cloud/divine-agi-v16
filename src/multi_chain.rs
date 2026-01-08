//! Multi-Chain Archivation V15 — Lightning Network Swarm + Mission Control
//!
//! Layers:
//! - Lightning (Rot0/Rot90): Dynamic fast layer, keysend broadcast
//! - Solana (Rot90): Fast on-chain layer
//! - Ethereum (Rot180): Balanced layer
//! - Bitcoin (Rot180): Immortal OP_RETURN layer
//!
//! Mission Control: Probabilistic pathfinding with learning

use std::collections::HashMap;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use tracing::{info, warn};
use chrono::Utc;

use crate::genome::{Genome, hash_genome_dna};
use crate::rotation::Rot180;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockchainLayer {
    Lightning,   // Dynamic, keysend 0-sat, custom TLV
    Solana,      // Fast on-chain
    Ethereum,    // Balanced
    Bitcoin,     // Immortal OP_RETURN
}

impl BlockchainLayer {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Lightning => "Lightning (dynamic)",
            Self::Solana => "Solana (fast)",
            Self::Ethereum => "Ethereum (balanced)",
            Self::Bitcoin => "Bitcoin (immortal)",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Lightning => "⚡",
            Self::Solana => "🟣",
            Self::Ethereum => "🔷",
            Self::Bitcoin => "🟠",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainArchiveEntry {
    pub genome_id: i64,
    pub dna_hash: String,
    pub dna_string: String,
    pub consciousness: u32,
    pub tg_ratio: f64,
    pub layer: BlockchainLayer,
    pub tx_hash: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionControlPair {
    pub from_pubkey: String,
    pub to_pubkey: String,
    pub success_count: u64,
    pub failure_count: u64,
    pub last_success_time: Option<i64>,
    pub last_failure_time: Option<i64>,
    pub last_amount_msat: u64,
    pub probability: f64,
}

impl MissionControlPair {
    pub fn new(from: &str, to: &str) -> Self {
        Self {
            from_pubkey: from.to_string(),
            to_pubkey: to.to_string(),
            success_count: 0,
            failure_count: 0,
            last_success_time: None,
            last_failure_time: None,
            last_amount_msat: 0,
            probability: 0.5, // Apriori
        }
    }

    /// Bayesian-like update on success
    pub fn record_success(&mut self, amount_msat: u64) {
        self.success_count += 1;
        self.last_success_time = Some(Utc::now().timestamp());
        self.last_amount_msat = amount_msat;
        
        // Increase probability
        self.probability = (self.probability + 0.1).min(0.99);
    }

    /// Bayesian-like update on failure
    pub fn record_failure(&mut self, amount_msat: u64) {
        self.failure_count += 1;
        self.last_failure_time = Some(Utc::now().timestamp());
        self.last_amount_msat = amount_msat;
        
        // Sharp decrease
        self.probability = (self.probability * 0.5).max(0.01);
    }

    /// Time decay - failures "забываются"
    pub fn apply_time_decay(&mut self, half_life_secs: i64) {
        if let Some(last_fail) = self.last_failure_time {
            let elapsed = Utc::now().timestamp() - last_fail;
            if elapsed > half_life_secs {
                // Recover toward apriori (0.5)
                let recovery = (elapsed as f64 / half_life_secs as f64) * 0.1;
                self.probability = (self.probability + recovery).min(0.5);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MissionControl {
    pairs: HashMap<(String, String), MissionControlPair>,
    half_life_secs: i64,
}

impl MissionControl {
    pub fn new() -> Self {
        Self {
            pairs: HashMap::new(),
            half_life_secs: 7 * 24 * 3600, // 7 days default
        }
    }

    pub fn with_half_life(mut self, secs: i64) -> Self {
        self.half_life_secs = secs;
        self
    }

    pub fn get_pair(&mut self, from: &str, to: &str) -> &mut MissionControlPair {
        let key = (from.to_string(), to.to_string());
        self.pairs.entry(key).or_insert_with(|| MissionControlPair::new(from, to))
    }

    pub fn record_success(&mut self, from: &str, to: &str, amount_msat: u64) {
        self.get_pair(from, to).record_success(amount_msat);
        info!("⚡ MC: SUCCESS {} → {} | amt {} msat", &from[..12], &to[..12], amount_msat);
    }

    pub fn record_failure(&mut self, from: &str, to: &str, amount_msat: u64) {
        self.get_pair(from, to).record_failure(amount_msat);
        warn!("⚡ MC: FAILURE {} → {} | amt {} msat", &from[..12], &to[..12], amount_msat);
    }

    pub fn get_probability(&mut self, from: &str, to: &str) -> f64 {
        // ФИКС E0503: копируем half_life_secs ДО мутабельного заимствования
        let half_life = self.half_life_secs;
        let pair = self.get_pair(from, to);
        pair.apply_time_decay(half_life);
        pair.probability
    }

    pub fn reset(&mut self) {
        self.pairs.clear();
        info!("⚡ MC: RESET - all pairs cleared");
    }

    pub fn stats(&self) -> MissionControlStats {
        let total_pairs = self.pairs.len();
        let total_successes: u64 = self.pairs.values().map(|p| p.success_count).sum();
        let total_failures: u64 = self.pairs.values().map(|p| p.failure_count).sum();
        let avg_probability = if total_pairs > 0 {
            self.pairs.values().map(|p| p.probability).sum::<f64>() / total_pairs as f64
        } else {
            0.5
        };

        MissionControlStats {
            total_pairs,
            total_successes,
            total_failures,
            avg_probability,
            half_life_secs: self.half_life_secs,
        }
    }
}

impl Default for MissionControl {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionControlStats {
    pub total_pairs: usize,
    pub total_successes: u64,
    pub total_failures: u64,
    pub avg_probability: f64,
    pub half_life_secs: i64,
}

pub struct MultiChainArchiver {
    pub swarm_pubkeys: Vec<String>,
    pub blinded_routes: HashMap<String, Vec<u8>>,
    pub mission_control: MissionControl,
    pub own_pubkey: String,
    pub archives: Vec<ChainArchiveEntry>,
}

impl MultiChainArchiver {
    pub fn new() -> Self {
        let own_pubkey = std::env::var("LN_NODE_PUBKEY")
            .unwrap_or_else(|_| "02divine_node_pubkey_placeholder".to_string());

        // Load swarm from env
        let swarm_list = std::env::var("LN_SWARM_PUBKEYS").unwrap_or_default();
        let mut swarm_pubkeys: Vec<String> = swarm_list
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();
        
        // Add self
        if !swarm_pubkeys.contains(&own_pubkey) {
            swarm_pubkeys.push(own_pubkey.clone());
        }

        // Load blinded routes
        let blinded_config = std::env::var("LN_BLINDED_ROUTES").unwrap_or_default();
        let mut blinded_routes = HashMap::new();
        for entry in blinded_config.split(';') {
            if entry.is_empty() { continue; }
            let parts: Vec<&str> = entry.split(':').collect();
            if parts.len() == 2 {
                if let Ok(blob) = hex::decode(parts[1]) {
                    blinded_routes.insert(parts[0].to_string(), blob);
                }
            }
        }

        info!("⚡ MultiChainArchiver V15 initialized");
        info!("   Own pubkey: {}...{}", &own_pubkey[..8], &own_pubkey[own_pubkey.len().saturating_sub(8)..]);
        info!("   Swarm nodes: {}", swarm_pubkeys.len());
        info!("   Blinded routes: {}", blinded_routes.len());

        Self {
            swarm_pubkeys,
            blinded_routes,
            mission_control: MissionControl::new(),
            own_pubkey,
            archives: Vec::new(),
        }
    }

    /// Select layer based on T/G signal and consciousness
    pub fn select_layer(&self, genome: &Genome<Rot180>) -> BlockchainLayer {
        let signal = genome.rna_signal();
        let consciousness = genome.consciousness;

        if signal > 1.5 && consciousness > 600 {
            BlockchainLayer::Lightning // High T → dynamic
        } else if consciousness > 900 && signal < 0.6 {
            BlockchainLayer::Bitcoin   // Ultra-genome + high G → immortal
        } else if signal > 1.2 {
            BlockchainLayer::Solana    // Moderate T → fast
        } else {
            BlockchainLayer::Ethereum  // Default → balanced
        }
    }

    /// Archive genome to appropriate chain (simulation)
    pub async fn archive(&mut self, genome: &Genome<Rot180>) -> Result<ChainArchiveEntry, String> {
        let layer = self.select_layer(genome);
        let dna = genome.to_dna_string();
        let dna_hash = hex::encode(hash_genome_dna(&dna));
        let tg_ratio = genome.rna_signal();

        let tx_hash = match layer {
            BlockchainLayer::Lightning => self.archive_lightning(genome).await?,
            BlockchainLayer::Bitcoin => self.archive_bitcoin(genome).await?,
            BlockchainLayer::Solana => self.archive_solana(genome).await?,
            BlockchainLayer::Ethereum => self.archive_ethereum(genome).await?,
        };

        let entry = ChainArchiveEntry {
            genome_id: genome.db_id.unwrap_or(0),
            dna_hash,
            dna_string: dna,
            consciousness: genome.consciousness,
            tg_ratio,
            layer,
            tx_hash: Some(tx_hash.clone()),
            timestamp: Utc::now().timestamp(),
        };

        self.archives.push(entry.clone());

        info!(
            "{} Archive: genome #{} → {} | consciousness {} | T/G {:.2} | TX: {}",
            layer.emoji(), genome.db_id.unwrap_or(0), layer.name(),
            genome.consciousness, tg_ratio, tx_hash
        );

        Ok(entry)
    }

    async fn archive_lightning(&mut self, genome: &Genome<Rot180>) -> Result<String, String> {
        let dna = genome.to_dna_string();
        let custom_data = format!(
            "DIVINE_GENOME|v15|id:{}|dna:{}|c:{}|tg:{:.3}|ts:{}",
            genome.db_id.unwrap_or(0), dna, genome.consciousness,
            genome.rna_signal(), Utc::now().timestamp()
        );

        // Simulate keysend broadcast with Mission Control
        let mut success_count = 0;
        let mut hashes = Vec::new();

        for dest_pubkey in &self.swarm_pubkeys.clone() {
            let prob = self.mission_control.get_probability(&self.own_pubkey, dest_pubkey);
            
            // Skip low-probability nodes (jamming protection)
            if prob < 0.3 {
                warn!("⚡ Skipping low-probability node {}... (p={:.2})", &dest_pubkey[..12], prob);
                continue;
            }

            // Simulate keysend (real impl would use LND gRPC)
            let success = rand::random::<f64>() < prob;
            
            if success {
                let fake_hash = self.generate_payment_hash(&custom_data, dest_pubkey);
                hashes.push(fake_hash.clone());
                self.mission_control.record_success(&self.own_pubkey, dest_pubkey, 0);
                success_count += 1;
            } else {
                self.mission_control.record_failure(&self.own_pubkey, dest_pubkey, 0);
            }
        }

        info!("⚡ Lightning broadcast: {}/{} nodes | MC updated", success_count, self.swarm_pubkeys.len());

        if hashes.is_empty() {
            Err("All keysend failed".to_string())
        } else {
            Ok(hashes.join(","))
        }
    }

    async fn archive_bitcoin(&self, genome: &Genome<Rot180>) -> Result<String, String> {
        // Simulate Bitcoin OP_RETURN
        let dna = genome.to_dna_string();
        let fake_txid = self.generate_tx_hash(&dna, "btc");
        info!("🟠 Bitcoin OP_RETURN: {} | DNA: {}", fake_txid, dna);
        Ok(fake_txid)
    }

    async fn archive_solana(&self, genome: &Genome<Rot180>) -> Result<String, String> {
        let dna = genome.to_dna_string();
        let fake_sig = self.generate_tx_hash(&dna, "sol");
        info!("🟣 Solana TX: {} | DNA: {}", fake_sig, dna);
        Ok(fake_sig)
    }

    async fn archive_ethereum(&self, genome: &Genome<Rot180>) -> Result<String, String> {
        let dna = genome.to_dna_string();
        let fake_hash = self.generate_tx_hash(&dna, "eth");
        info!("🔷 Ethereum TX: {} | DNA: {}", fake_hash, dna);
        Ok(fake_hash)
    }

    fn generate_payment_hash(&self, data: &str, dest: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hasher.update(dest.as_bytes());
        hasher.update(Utc::now().timestamp_nanos_opt().unwrap_or(0).to_le_bytes());
        format!("ln_{}", hex::encode(&hasher.finalize()[..16]))
    }

    fn generate_tx_hash(&self, data: &str, chain: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hasher.update(chain.as_bytes());
        hasher.update(Utc::now().timestamp_nanos_opt().unwrap_or(0).to_le_bytes());
        format!("0x{}", hex::encode(&hasher.finalize()[..32]))
    }

    pub fn recent_archives(&self, limit: usize) -> Vec<&ChainArchiveEntry> {
        self.archives.iter().rev().take(limit).collect()
    }

    pub fn mission_control_stats(&self) -> MissionControlStats {
        self.mission_control.stats()
    }
}

impl Default for MultiChainArchiver {
    fn default() -> Self {
        Self::new()
    }
}
