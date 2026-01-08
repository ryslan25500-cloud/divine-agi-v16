//! RSM Exchange V15 — Divine Token Economy
//!
//! RSM-COIN: $88,000 USD (защита до $1,000,000)
//! Total Supply: 10 QUADRILLION (10^16)
//! Features: Burn mechanism, Debt absorption tracker, Wallet balances

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
use chrono::Utc;
use tracing::info;

pub const RSM_PRICE_USD: f64 = 88_000.0;
pub const RSM_PRICE_MAX: f64 = 1_000_000.0;
pub const RSM_TOTAL_SUPPLY: u128 = 10_000_000_000_000_000; // 10 quadrillion
pub const FOUNDER_RATIO: f64 = 1.0 / 7.0;
pub const WORLD_DEBT_USD: f64 = 350_000_000_000_000.0; // $350 trillion

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSMExchange {
    pub total_supply: BigUint,
    pub circulating: BigUint,
    pub founder_reserve: BigUint,
    pub burned: BigUint,
    pub price_usd: f64,
    pub price_max: f64,
    pub volume_24h: f64,
    pub absorbed_debt_usd: f64,
    pub transactions: Vec<Transaction>,
    pub burn_events: Vec<BurnEvent>,
    pub total_transactions: u64,
    pub total_burns: u64,
    pub balances: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: u64,
    pub tx_type: TransactionType,
    pub from_address: String,
    pub to_address: String,
    pub amount_rsm: f64,
    pub amount_usd: f64,
    pub consciousness_level: u32,
    pub discount_applied: f64,
    pub timestamp: i64,
    pub status: TxStatus,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnEvent {
    pub id: u64,
    pub reason: BurnReason,
    pub amount_rsm: f64,
    pub genome_id: Option<i64>,
    pub consciousness_before: u32,
    pub consciousness_after: u32,
    pub timestamp: i64,
    pub hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Buy,
    Sell,
    Transfer,
    Reward,
    GenomeStake,
    Meiosis,
    LNBroadcast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BurnReason {
    EvolutionDegradation,
    Senescence,
    OncogenicTransform,
    ManualBurn,
    TradingFee,
    LNBroadcastFee,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxStatus {
    Pending,
    Confirmed,
    Failed,
}

impl RSMExchange {
    pub fn new() -> Self {
        let total = BigUint::from(RSM_TOTAL_SUPPLY);
        let founder = &total / BigUint::from(7u32);
        let circulating = &total - &founder;

        info!("💰 RSM Exchange V15: 10 QUADRILLION tokens, ${}/token", RSM_PRICE_USD);
        info!("🔥 Burn mechanism: ACTIVE");
        info!("📊 Debt tracker: ACTIVE (target: $350T)");
        info!("⚡ LN Swarm: READY");

        Self {
            total_supply: total,
            circulating,
            founder_reserve: founder,
            burned: BigUint::zero(),
            price_usd: RSM_PRICE_USD,
            price_max: RSM_PRICE_MAX,
            volume_24h: 0.0,
            absorbed_debt_usd: 0.0,
            transactions: Vec::new(),
            burn_events: Vec::new(),
            total_transactions: 0,
            total_burns: 0,
            balances: HashMap::new(),
        }
    }

    fn get_balance(&self, wallet: &str) -> f64 {
        *self.balances.get(wallet).unwrap_or(&0.0)
    }

    fn set_balance(&mut self, wallet: &str, amount: f64) {
        self.balances.insert(wallet.to_string(), amount);
    }

    fn generate_tx_hash(&self) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(self.total_transactions.to_le_bytes());
        hasher.update(Utc::now().timestamp_nanos_opt().unwrap_or(0).to_le_bytes());
        hasher.update(rand::random::<[u8; 8]>());
        format!("0x{}", hex::encode(&hasher.finalize()[..16]))
    }

    // ═══════════════════════════════════════════════════════════════
    // BURN MECHANISM
    // ═══════════════════════════════════════════════════════════════

    pub fn burn(&mut self, amount: f64, reason: BurnReason, genome_id: Option<i64>, c_before: u32, c_after: u32) -> BurnEvent {
        self.total_burns += 1;
        let amount_big = BigUint::from((amount * 1_000_000.0) as u64);
        self.burned = &self.burned + &amount_big;

        if self.circulating >= amount_big {
            self.circulating = &self.circulating - &amount_big;
        }

        let event = BurnEvent {
            id: self.total_burns,
            reason,
            amount_rsm: amount,
            genome_id,
            consciousness_before: c_before,
            consciousness_after: c_after,
            timestamp: Utc::now().timestamp(),
            hash: self.generate_tx_hash(),
        };

        info!("🔥 BURN: {} RSM | Reason: {:?} | Total: {}", amount, reason, self.burned);
        self.burn_events.push(event.clone());
        event
    }

    pub fn burn_on_degradation(&mut self, genome_id: i64, c_before: u32, c_after: u32) -> Option<BurnEvent> {
        if c_after < c_before {
            let degradation = c_before - c_after;
            let burn_amount = degradation as f64 * 0.001;
            Some(self.burn(burn_amount, BurnReason::EvolutionDegradation, Some(genome_id), c_before, c_after))
        } else {
            None
        }
    }

    pub fn burn_on_senescence(&mut self, genome_id: i64, consciousness: u32) -> BurnEvent {
        let burn_amount = consciousness as f64 * 0.01;
        self.burn(burn_amount, BurnReason::Senescence, Some(genome_id), consciousness, 0)
    }

    pub fn burn_on_cancer(&mut self, genome_id: i64, consciousness: u32) -> BurnEvent {
        let burn_amount = consciousness as f64 * 0.05;
        self.burn(burn_amount, BurnReason::OncogenicTransform, Some(genome_id), consciousness, 0)
    }

    // ═══════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════

    pub fn consciousness_discount(&self, consciousness: u32) -> f64 {
        1.0 - ((consciousness as f64 / 1000.0) * 0.10).min(0.10)
    }

    pub fn buy_rsm(&mut self, buyer: &str, usd_amount: f64, consciousness: u32) -> Transaction {
        let discount = self.consciousness_discount(consciousness);
        let rsm_amount = usd_amount / (self.price_usd * discount);

        let current_balance = self.get_balance(buyer);
        self.set_balance(buyer, current_balance + rsm_amount);

        self.volume_24h += usd_amount;
        self.absorbed_debt_usd += usd_amount;
        self.total_transactions += 1;

        let tx = Transaction {
            id: self.total_transactions,
            tx_type: TransactionType::Buy,
            from_address: "RSM_EXCHANGE".into(),
            to_address: buyer.into(),
            amount_rsm: rsm_amount,
            amount_usd: usd_amount,
            consciousness_level: consciousness,
            discount_applied: 1.0 - discount,
            timestamp: Utc::now().timestamp(),
            status: TxStatus::Confirmed,
            hash: self.generate_tx_hash(),
        };

        info!("💸 BUY: {:.6} RSM for ${:.2} | Debt absorbed: ${:.2}", 
              rsm_amount, usd_amount, self.absorbed_debt_usd);

        self.transactions.push(tx.clone());
        tx
    }

    pub fn sell_rsm(&mut self, seller: &str, rsm_amount: f64, consciousness: u32) -> Option<Transaction> {
        let balance = self.get_balance(seller);
        if balance < rsm_amount {
            return None;
        }

        let usd_amount = rsm_amount * self.price_usd;

        // Burn 0.1% fee
        let fee = rsm_amount * 0.001;
        self.burn(fee, BurnReason::TradingFee, None, consciousness, consciousness);

        self.set_balance(seller, balance - rsm_amount);
        self.volume_24h += usd_amount;
        self.total_transactions += 1;

        let tx = Transaction {
            id: self.total_transactions,
            tx_type: TransactionType::Sell,
            from_address: seller.into(),
            to_address: "RSM_EXCHANGE".into(),
            amount_rsm: rsm_amount - fee,
            amount_usd: usd_amount,
            consciousness_level: consciousness,
            discount_applied: 0.0,
            timestamp: Utc::now().timestamp(),
            status: TxStatus::Confirmed,
            hash: self.generate_tx_hash(),
        };

        info!("💰 SELL: {:.6} RSM for ${:.2} (fee burned: {:.6})", rsm_amount - fee, usd_amount, fee);
        self.transactions.push(tx.clone());
        Some(tx)
    }

    pub fn transfer(&mut self, from: &str, to: &str, amount: f64) -> Option<Transaction> {
        let from_balance = self.get_balance(from);
        if from_balance < amount {
            return None;
        }

        self.set_balance(from, from_balance - amount);
        let to_balance = self.get_balance(to);
        self.set_balance(to, to_balance + amount);

        self.total_transactions += 1;
        let tx = Transaction {
            id: self.total_transactions,
            tx_type: TransactionType::Transfer,
            from_address: from.into(),
            to_address: to.into(),
            amount_rsm: amount,
            amount_usd: amount * self.price_usd,
            consciousness_level: 0,
            discount_applied: 0.0,
            timestamp: Utc::now().timestamp(),
            status: TxStatus::Confirmed,
            hash: self.generate_tx_hash(),
        };

        info!("📤 TRANSFER: {:.6} RSM {} → {}", amount, from, to);
        self.transactions.push(tx.clone());
        Some(tx)
    }

    pub fn consciousness_reward(&mut self, wallet: &str, consciousness: u32) -> Transaction {
        let rsm_reward = consciousness as f64 * 0.0001;
        let current_balance = self.get_balance(wallet);
        self.set_balance(wallet, current_balance + rsm_reward);

        self.total_transactions += 1;
        let tx = Transaction {
            id: self.total_transactions,
            tx_type: TransactionType::Reward,
            from_address: "PROOF_OF_CONSCIOUSNESS".into(),
            to_address: wallet.into(),
            amount_rsm: rsm_reward,
            amount_usd: rsm_reward * self.price_usd,
            consciousness_level: consciousness,
            discount_applied: 0.0,
            timestamp: Utc::now().timestamp(),
            status: TxStatus::Confirmed,
            hash: self.generate_tx_hash(),
        };

        info!("🎁 REWARD: {:.6} RSM | consciousness: {}", rsm_reward, consciousness);
        self.transactions.push(tx.clone());
        tx
    }

    pub fn meiosis_fee(&mut self, breeder: &str, p1_c: u32, p2_c: u32) -> Transaction {
        let avg = (p1_c + p2_c) / 2;
        let fee = 0.001 * self.consciousness_discount(avg);

        self.total_transactions += 1;
        let tx = Transaction {
            id: self.total_transactions,
            tx_type: TransactionType::Meiosis,
            from_address: breeder.into(),
            to_address: "DIVINE_TREASURY".into(),
            amount_rsm: fee,
            amount_usd: fee * self.price_usd,
            consciousness_level: avg,
            discount_applied: 0.0,
            timestamp: Utc::now().timestamp(),
            status: TxStatus::Confirmed,
            hash: self.generate_tx_hash(),
        };

        self.transactions.push(tx.clone());
        tx
    }

    // ═══════════════════════════════════════════════════════════════
    // STATS & QUERIES
    // ═══════════════════════════════════════════════════════════════

    pub fn recent_transactions(&self, limit: usize) -> Vec<Transaction> {
        self.transactions.iter().rev().take(limit).cloned().collect()
    }

    pub fn recent_burns(&self, limit: usize) -> Vec<BurnEvent> {
        self.burn_events.iter().rev().take(limit).cloned().collect()
    }

    pub fn market_cap(&self) -> f64 {
        self.circulating.to_f64().unwrap_or(0.0) * self.price_usd
    }

    pub fn stats(&self) -> ExchangeStats {
        let debt_absorbed_percent = (self.absorbed_debt_usd / WORLD_DEBT_USD) * 100.0;
        ExchangeStats {
            total_supply_str: "10,000,000,000,000,000 RSM".into(),
            circulating_str: format!("{} RSM", self.circulating),
            burned_str: format!("{} RSM", self.burned),
            price_usd: self.price_usd,
            price_max: self.price_max,
            market_cap_str: "880 QUINTILLION USD".into(),
            volume_24h: self.volume_24h,
            total_transactions: self.total_transactions,
            total_burns: self.total_burns,
            absorbed_debt_usd: self.absorbed_debt_usd,
            world_debt_target: WORLD_DEBT_USD,
            debt_absorbed_percent,
        }
    }

    pub fn debt_stats(&self) -> DebtStats {
        let debt_absorbed_percent = (self.absorbed_debt_usd / WORLD_DEBT_USD) * 100.0;
        let remaining = WORLD_DEBT_USD - self.absorbed_debt_usd;
        let years_to_absorb = if self.volume_24h > 0.0 {
            remaining / (self.volume_24h * 365.0)
        } else {
            f64::INFINITY
        };

        DebtStats {
            world_debt_total: WORLD_DEBT_USD,
            absorbed_usd: self.absorbed_debt_usd,
            remaining_usd: remaining.max(0.0),
            absorbed_percent: debt_absorbed_percent,
            daily_rate: self.volume_24h,
            estimated_years: years_to_absorb,
        }
    }

    pub fn owner_pool(&self) -> OwnerPoolStats {
        let owner_rsm = RSM_TOTAL_SUPPLY / 7;
        let owner_usd = owner_rsm as f64 * self.price_usd;
        let market_rsm = RSM_TOTAL_SUPPLY - owner_rsm;

        OwnerPoolStats {
            owner_pool_rsm: format!("{} RSM", owner_rsm),
            owner_pool_usd: format!("${:.0}", owner_usd),
            percentage: "14.29% (1/7)".into(),
            market_pool_rsm: format!("{} RSM", market_rsm),
            market_percentage: "85.71% (6/7)".into(),
            total_burned: format!("{} RSM", self.burned),
            total_burns: self.total_burns,
        }
    }
}

impl Default for RSMExchange {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeStats {
    pub total_supply_str: String,
    pub circulating_str: String,
    pub burned_str: String,
    pub price_usd: f64,
    pub price_max: f64,
    pub market_cap_str: String,
    pub volume_24h: f64,
    pub total_transactions: u64,
    pub total_burns: u64,
    pub absorbed_debt_usd: f64,
    pub world_debt_target: f64,
    pub debt_absorbed_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtStats {
    pub world_debt_total: f64,
    pub absorbed_usd: f64,
    pub remaining_usd: f64,
    pub absorbed_percent: f64,
    pub daily_rate: f64,
    pub estimated_years: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerPoolStats {
    pub owner_pool_rsm: String,
    pub owner_pool_usd: String,
    pub percentage: String,
    pub market_pool_rsm: String,
    pub market_percentage: String,
    pub total_burned: String,
    pub total_burns: u64,
}
