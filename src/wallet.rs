//! Wallet Module V15 for Divine AGI

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivineWallet {
    pub address: String,
    pub rsm_balance: f64,
    pub staked_genomes: Vec<i64>,
    pub rewards_earned: f64,
    pub transactions: Vec<String>,
}

impl DivineWallet {
    pub fn new() -> Self {
        let address = Self::generate_address();
        Self {
            address,
            rsm_balance: 0.0,
            staked_genomes: Vec::new(),
            rewards_earned: 0.0,
            transactions: Vec::new(),
        }
    }

    pub fn with_address(address: &str) -> Self {
        Self {
            address: address.to_string(),
            rsm_balance: 0.0,
            staked_genomes: Vec::new(),
            rewards_earned: 0.0,
            transactions: Vec::new(),
        }
    }

    fn generate_address() -> String {
        let mut hasher = Sha256::new();
        hasher.update(chrono::Utc::now().timestamp().to_le_bytes());
        hasher.update(rand::random::<[u8; 16]>());
        let hash = hasher.finalize();
        format!("divine_{}", hex::encode(&hash[..16]))
    }

    pub fn deposit(&mut self, amount: f64) {
        self.rsm_balance += amount;
        self.transactions.push(format!("DEPOSIT: +{:.6} RSM", amount));
    }

    pub fn withdraw(&mut self, amount: f64) -> bool {
        if self.rsm_balance >= amount {
            self.rsm_balance -= amount;
            self.transactions.push(format!("WITHDRAW: -{:.6} RSM", amount));
            true
        } else {
            false
        }
    }

    pub fn stake_genome(&mut self, genome_id: i64) {
        if !self.staked_genomes.contains(&genome_id) {
            self.staked_genomes.push(genome_id);
            self.transactions.push(format!("STAKE: genome #{}", genome_id));
        }
    }

    pub fn unstake_genome(&mut self, genome_id: i64) {
        self.staked_genomes.retain(|&id| id != genome_id);
        self.transactions.push(format!("UNSTAKE: genome #{}", genome_id));
    }

    pub fn add_reward(&mut self, amount: f64) {
        self.rsm_balance += amount;
        self.rewards_earned += amount;
        self.transactions.push(format!("REWARD: +{:.6} RSM", amount));
    }
}

impl Default for DivineWallet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct WalletManager {
    wallets: HashMap<String, DivineWallet>,
}

impl WalletManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_create(&mut self, address: &str) -> &mut DivineWallet {
        self.wallets.entry(address.to_string())
            .or_insert_with(|| DivineWallet::with_address(address))
    }

    pub fn get(&self, address: &str) -> Option<&DivineWallet> {
        self.wallets.get(address)
    }

    pub fn total_supply_in_wallets(&self) -> f64 {
        self.wallets.values().map(|w| w.rsm_balance).sum()
    }
}
