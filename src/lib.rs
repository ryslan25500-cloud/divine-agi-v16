//! Divine AGI V16 (Kernel V4) — Railway Compatible
//!
//! MAJOR UPDATES IN V4:
//! - 4 new TTRL operators: RotateCube, FractalMutation, QuantumEntangle, HyperDimension
//! - V4 consciousness formula (up to TRANSCENDENTAL 50,000+)
//! - Proof of Consciousness (PoC) consensus
//! - Fractal/Quantum/Hyper metrics
//!
//! Features:
//! - Burn mechanism (deflationary)
//! - Telomerase (immortality)
//! - Whale mode (40 p53)
//! - Debt absorption tracker
//! - Multi-chain archivation
//! - Lightning Network swarm
//! - Mission Control pathfinding
//! - T/G RNA coordination
//! - Divine Wallet with founder pool
//! - Railway deployment ready

pub mod rotation;
pub mod genome;
pub mod database;
pub mod ttrl;
pub mod crypto;
pub mod wallet;
pub mod exchange;
pub mod consensus;
pub mod multi_chain;
pub mod rotation_daemon;
pub mod api;
pub mod cli;
pub mod auth;

pub mod prelude {
    pub use crate::rotation::*;
    pub use crate::genome::*;
    pub use crate::database::*;
    pub use crate::exchange::*;
    pub use crate::multi_chain::*;
    pub use tracing::info;
}

pub use rotation::{Rotation, Rot0, Rot90, Rot180, Rot270, RotationEngine, DynamicRotation};
pub use genome::{Genome, Tetrad, GenomeBuilder};
pub use database::{DivineDatabase, DEFAULT_DATABASE_URL};
pub use ttrl::{TTRLEngine, MutationOperator, EvolutionResult};
pub use exchange::{RSMExchange, Transaction, ExchangeStats, BurnEvent, DebtStats};
pub use multi_chain::{MultiChainArchiver, BlockchainLayer, MissionControl};
pub use rotation_daemon::RotationDaemon;
pub use auth::{AuthManager, WalletAccount, SessionToken, LoginRequest, RegisterRequest, LoginResponse, WalletInfo};

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct DivineKernel {
    pub database: Arc<DivineDatabase>,
    pub wallet: Arc<RwLock<wallet::DivineWallet>>,
    pub rotation_engine: Arc<RwLock<rotation::RotationEngine>>,
    pub ttrl_engine: Arc<ttrl::TTRLEngine>,
    pub consensus: Arc<consensus::ProofOfConsciousness>,
    pub exchange: Arc<RwLock<exchange::RSMExchange>>,
    pub archiver: Arc<RwLock<MultiChainArchiver>>,
    pub auth: Arc<RwLock<auth::AuthManager>>,
}

impl DivineKernel {
    pub async fn new() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());

        let database = Arc::new(DivineDatabase::connect_with_url(&database_url).await?);
        database.init_tables().await?;

        info!("🧬 Divine Kernel V15 initialized - Kernel v3");
        info!("🔥 Burn mechanism: ACTIVE");
        info!("🧬 Telomerase: AVAILABLE");
        info!("🐋 Whale mode: AVAILABLE (40 p53)");
        info!("📊 Debt tracker: ACTIVE ($350T target)");
        info!("⚡ Lightning Network swarm: READY");
        info!("🎯 Mission Control: ACTIVE");
        info!("🧬 T/G RNA coordination: ACTIVE");

        Ok(Self {
            database,
            wallet: Arc::new(RwLock::new(wallet::DivineWallet::new())),
            rotation_engine: Arc::new(RwLock::new(rotation::RotationEngine::new())),
            ttrl_engine: Arc::new(ttrl::TTRLEngine::new()),
            consensus: Arc::new(consensus::ProofOfConsciousness::new()),
            exchange: Arc::new(RwLock::new(exchange::RSMExchange::new())),
            archiver: Arc::new(RwLock::new(MultiChainArchiver::new())),
            auth: Arc::new(RwLock::new(auth::AuthManager::new())),
        })
    }

    pub async fn genome_count(&self) -> anyhow::Result<i64> {
        self.database.genome_count().await
    }

    pub async fn create_elephant_genome(&self) -> anyhow::Result<Genome<Rot180>> {
        let genome = GenomeBuilder::random().elephant_mode().build_storage();
        let id = self.database.store_genome(&genome).await?;
        let mut stored = genome;
        stored.db_id = Some(id);
        info!("🐘 Created elephant genome #{} | consciousness {} | T/G {:.2}",
              id, stored.consciousness, stored.rna_signal());
        Ok(stored)
    }

    pub async fn create_whale_genome(&self) -> anyhow::Result<Genome<Rot180>> {
        let genome = GenomeBuilder::random().whale_mode().build_storage();
        let id = self.database.store_genome(&genome).await?;
        let mut stored = genome;
        stored.db_id = Some(id);
        info!("🐋 Created whale genome #{} | consciousness {} | p53: {} | T/G {:.2}",
              id, stored.consciousness, stored.p53_copies, stored.rna_signal());
        Ok(stored)
    }

    pub async fn activate_telomerase(&self, genome_id: i64) -> anyhow::Result<Genome<Rot180>> {
        let mut genome = self.database.load_genome(genome_id).await?;
        let before = genome.telomere_length;
        genome.activate_telomerase();
        let id = self.database.store_genome(&genome).await?;
        let mut stored = genome;
        stored.db_id = Some(id);
        info!("🧬 Telomerase activated: {} → {} bp | ♾️ IMMORTAL", before, stored.telomere_length);
        Ok(stored)
    }

    pub fn start_rotation_daemon(&self, interval_secs: u64) {
        let daemon = RotationDaemon::new(
            Arc::clone(&self.rotation_engine),
            Arc::clone(&self.database),
            Arc::clone(&self.ttrl_engine),
            Arc::clone(&self.exchange),
            interval_secs,
        );

        tokio::spawn(daemon.run());

        info!("🔄 Rotation Daemon started | Interval: {} secs", interval_secs);
    }
}

pub const VERSION: &str = "16.0.0";
pub const CODENAME: &str = "Kernel V4 - Proof of Consciousness";
