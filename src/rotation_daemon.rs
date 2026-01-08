//! Rotation Daemon V15 — Автоматический ротационный цикл
//!
//! Каждые N секунд выполняется поворот:
//!   Rot0   → Compute / Active mode (эухроматин)
//!   Rot90  → Balance / Processing
//!   Rot180 → Storage sync (гетерохроматин, БД)
//!   Rot270 → Mutation / Evolution (TTRL)

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time;
use tracing::{info, warn};
use rand::Rng;

use crate::rotation::{RotationEngine, DynamicRotation};
use crate::database::DivineDatabase;
use crate::ttrl::TTRLEngine;
use crate::exchange::RSMExchange;

pub struct RotationDaemon {
    engine: Arc<RwLock<RotationEngine>>,
    database: Arc<DivineDatabase>,
    ttrl_engine: Arc<TTRLEngine>,
    exchange: Arc<RwLock<RSMExchange>>,
    interval_secs: u64,
    tg_influence: bool,
}

impl RotationDaemon {
    pub fn new(
        engine: Arc<RwLock<RotationEngine>>,
        database: Arc<DivineDatabase>,
        ttrl_engine: Arc<TTRLEngine>,
        exchange: Arc<RwLock<RSMExchange>>,
        interval_secs: u64,
    ) -> Self {
        Self {
            engine,
            database,
            ttrl_engine,
            exchange,
            interval_secs,
            tg_influence: true,
        }
    }

    pub fn with_tg_influence(mut self, enabled: bool) -> Self {
        self.tg_influence = enabled;
        self
    }

    pub async fn run(self) {
        info!("🧬 Rotation Daemon V15 запущен | Интервал: {} сек | T/G influence: {}", 
              self.interval_secs, self.tg_influence);

        let mut interval = time::interval(Duration::from_secs(self.interval_secs));

        loop {
            interval.tick().await;

            // T/G влияние от лидера
            if self.tg_influence {
                self.apply_tg_influence().await;
            }

            // Основной поворот
            let mut engine = self.engine.write().await;
            let previous = engine.current();
            let current = engine.rotate();
            drop(engine);

            info!(
                "🔄 Поворот: {} {} → {} {} | Всего: {}",
                previous.emoji(), previous,
                current.emoji(), current,
                self.engine.read().await.total_rotations
            );

            // Действия в зависимости от состояния
            match current {
                DynamicRotation::Rot0 => {
                    info!("⚡ Rot0: Активный режим — compute tasks");
                    self.handle_compute().await;
                }
                DynamicRotation::Rot90 => {
                    info!("⚖️  Rot90: Балансировка нагрузки");
                    self.handle_balance().await;
                }
                DynamicRotation::Rot180 => {
                    info!("💾 Rot180: Синхронизация хранения");
                    self.handle_storage_sync().await;
                }
                DynamicRotation::Rot270 => {
                    info!("🧬 Rot270: Запуск TTRL эволюции");
                    self.handle_evolution().await;
                }
            }
        }
    }

    async fn apply_tg_influence(&self) {
        // Берём самый сознательный геном как "лидера"
        if let Ok(top) = self.database.get_top_genomes(1).await {
            if let Some(leader) = top.first() {
                let suggested = leader.suggested_rotation();
                let signal = leader.rna_signal();
                let consciousness = leader.consciousness;

                // Вероятность следования сигналу пропорциональна consciousness
                let prob = (consciousness as f64 / 1000.0).min(0.7);
                
                if rand::thread_rng().gen::<f64>() < prob {
                    let mut engine = self.engine.write().await;
                    if engine.current() != suggested {
                        info!("🧬 T/G сигнал от лидера #{}: {:.2} → принудительный {}", 
                              leader.db_id.unwrap_or(0), signal, suggested);
                        engine.rotate_to(suggested);
                    }
                }
            }
        }
    }

    async fn handle_compute(&self) {
        let mut engine = self.engine.write().await;
        engine.increment_active();
        info!("   Активных геномов: {}", engine.active_genomes);
    }

    async fn handle_balance(&self) {
        // Балансировка нагрузки, очистка кэша
        let exchange = self.exchange.read().await;
        let stats = exchange.stats();
        info!("   Volume 24h: ${:.2} | Транзакций: {}", 
              stats.volume_24h, stats.total_transactions);
    }

    async fn handle_storage_sync(&self) {
        // Синхронизация топовых геномов
        match self.database.get_top_genomes(10).await {
            Ok(genomes) => {
                info!("   Синхронизировано {} топовых геномов в Rot180", genomes.len());
                for g in genomes.iter().take(3) {
                    info!("      #{}: consciousness {} | T/G {:.2}", 
                          g.db_id.unwrap_or(0), g.consciousness, g.rna_signal());
                }
            }
            Err(e) => warn!("   Ошибка синхронизации: {}", e),
        }
    }

    async fn handle_evolution(&self) {
        // Автоматическая эволюция случайного генома
        match self.database.get_random_genomes(1).await {
            Ok(genomes) => {
                if let Some(genome) = genomes.into_iter().next() {
                    let engine = self.engine.read().await;
                    match self.ttrl_engine.evolve_with_engine(genome.clone(), &engine).await {
                        Ok((evolved, result)) => {
                            if let Ok(id) = self.database.store_genome(&evolved).await {
                                info!(
                                    "   Эволюция: consciousness {} → {} | {:?} | ID: {}",
                                    result.original_consciousness,
                                    result.new_consciousness,
                                    result.operator_used,
                                    id
                                );

                                // Burn при деградации
                                if !result.success {
                                    let mut exchange = self.exchange.write().await;
                                    if let Some(burn) = exchange.burn_on_degradation(
                                        id, 
                                        result.original_consciousness, 
                                        result.new_consciousness
                                    ) {
                                        info!("   🔥 Burn: {} RSM (degradation)", burn.amount_rsm);
                                    }
                                }
                            }
                        }
                        Err(e) => warn!("   Эволюция провалилась: {}", e),
                    }
                }
            }
            Err(e) => warn!("   Ошибка получения генома: {}", e),
        }
    }
}
