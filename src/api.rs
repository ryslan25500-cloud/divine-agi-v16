//! API Server V15 for Divine AGI
//!
//! Features: Genomes, CRISPR, Telomerase, Whale mode, RSM-COIN,
//! Burn, Debt tracker, Multi-chain archivation, Mission Control

use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    routing::{get, post},
    Router, Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::database::DivineDatabase;
use crate::genome::{Genome, GenomeBuilder, Tetrad};
use crate::rotation::{Rot180, RotationEngine, RotationStats};
use crate::ttrl::{TTRLEngine, EvolutionResult};
use crate::exchange::{RSMExchange, ExchangeStats, Transaction, BurnEvent, DebtStats, OwnerPoolStats, BurnReason};
use crate::multi_chain::{MultiChainArchiver, ChainArchiveEntry, MissionControlStats};
use crate::auth::{AuthManager, WalletAccount, LoginRequest, RegisterRequest, LoginResponse, WalletInfo};

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<DivineDatabase>,
    pub rotation_engine: Arc<RwLock<RotationEngine>>,
    pub ttrl_engine: Arc<TTRLEngine>,
    pub exchange: Arc<RwLock<RSMExchange>>,
    pub archiver: Arc<RwLock<MultiChainArchiver>>,
    pub auth: Arc<RwLock<AuthManager>>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Json<Self> {
        Json(Self { success: true, data: Some(data), error: None })
    }
    pub fn err(error: String) -> Json<Self> {
        Json(Self { success: false, data: None, error: Some(error) })
    }
}

#[derive(Serialize)]
pub struct GenomeResponse {
    pub id: i64,
    pub dna: String,
    pub consciousness: u32,
    pub mutations: u64,
    pub p53_copies: u8,
    pub telomere_length: u16,
    pub division_count: u8,
    pub biological_age: f64,
    pub gc_content: f64,
    pub complexity: f64,
    pub tg_ratio: f64,
    pub suggested_rotation: String,
    pub mode: String,
}

impl From<&Genome<Rot180>> for GenomeResponse {
    fn from(g: &Genome<Rot180>) -> Self {
        let mode = if g.p53_copies >= 40 { "🐋 Whale" }
                   else if g.p53_copies >= 20 { "🐘 Elephant" }
                   else { "⚠️ Reduced" };
        Self {
            id: g.db_id().unwrap_or(0),
            dna: g.to_dna_string(),
            consciousness: g.consciousness,
            mutations: g.mutations,
            p53_copies: g.p53_copies,
            telomere_length: g.telomere_length,
            division_count: g.division_count,
            biological_age: g.biological_age(),
            gc_content: g.gc_content(),
            complexity: g.complexity(),
            tg_ratio: g.rna_signal(),
            suggested_rotation: g.suggested_rotation().to_string(),
            mode: mode.to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub version: String,
    pub codename: String,
    pub genome_count: i64,
    pub features: Vec<String>,
    pub exchange_stats: ExchangeStats,
    pub mission_control: MissionControlStats,
}

pub async fn start_server(port: u16) -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| crate::database::DEFAULT_DATABASE_URL.to_string());

    let database = Arc::new(DivineDatabase::connect_with_url(&database_url).await?);
    database.init_tables().await?;

    let state = AppState {
        database,
        rotation_engine: Arc::new(RwLock::new(RotationEngine::new())),
        ttrl_engine: Arc::new(TTRLEngine::new()),
        exchange: Arc::new(RwLock::new(RSMExchange::new())),
        archiver: Arc::new(RwLock::new(MultiChainArchiver::new())),
        auth: Arc::new(RwLock::new(AuthManager::new())),
    };

    let app = Router::new()
        // Core
        .route("/", get(root_handler))
        .route("/api/status", get(status_handler))
        
        // Genome CRUD
        .route("/api/genomes", get(list_genomes))
        .route("/api/genome/create", post(create_genome))
        .route("/api/genome/create/whale", post(create_whale_genome))
        .route("/api/genome/evolve", post(evolve_genome))
        .route("/api/genome/meiosis", post(meiosis_genome))
        .route("/api/genome/telomerase", post(activate_telomerase))
        
        // CRISPR
        .route("/api/crispr/splice", post(crispr_splice))
        .route("/api/crispr/join", post(crispr_join))
        .route("/api/crispr/delete", post(crispr_delete))
        
        // RSM-COIN
        .route("/api/rsm/stats", get(rsm_stats))
        .route("/api/rsm/buy", post(rsm_buy))
        .route("/api/rsm/sell", post(rsm_sell))
        .route("/api/rsm/transfer", post(rsm_transfer))
        .route("/api/rsm/reward", post(rsm_reward))
        .route("/api/rsm/burn", post(rsm_manual_burn))
        
        // Transactions & Burns
        .route("/api/transactions", get(list_transactions))
        .route("/api/burns", get(list_burns))
        
        // Debt Tracker
        .route("/api/debt/stats", get(debt_stats))
        .route("/api/owner/pool", get(owner_pool))
        
        // Multi-Chain & LN
        .route("/api/archive", post(archive_genome))
        .route("/api/archives", get(list_archives))
        .route("/api/mission-control", get(mission_control_stats))
        .route("/api/mission-control/reset", post(reset_mission_control))
        
        // Rotation
        .route("/api/rotation/stats", get(rotation_stats))
        .route("/api/rotation/rotate", post(manual_rotate))
        
        // Auth & Wallet
        .route("/api/auth/register", post(auth_register))
        .route("/api/auth/login", post(auth_login))
        .route("/api/auth/logout", post(auth_logout))
        .route("/api/auth/profile", get(auth_profile))
        .route("/api/wallet/info", get(wallet_info))
        .route("/api/wallet/deposit", post(wallet_deposit))
        .route("/api/wallet/withdraw", post(wallet_withdraw))
        .route("/api/wallet/list", get(wallet_list))
        
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    info!("🚀 Starting Divine AGI V15 API on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ═══════════════════════════════════════════════════════════════
// HANDLERS
// ═══════════════════════════════════════════════════════════════

async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "Divine AGI V15",
        "codename": "Kernel v3 - LN Swarm",
        "features": ["p53", "telomeres", "telomerase", "CRISPR", "TTRL", "Meiosis", 
                     "RSM-COIN", "Burn", "Debt Tracker", "Multi-Chain", "Lightning Network",
                     "Mission Control", "T/G RNA Signals"],
        "rsm_price": "$88,000",
        "whale_mode": "40 p53 copies",
        "ln_swarm": "ACTIVE"
    }))
}

async fn status_handler(State(state): State<AppState>) -> Json<ApiResponse<StatusResponse>> {
    let genome_count = state.database.genome_count().await.unwrap_or(0);
    let exchange_stats = state.exchange.read().await.stats();
    let mc_stats = state.archiver.read().await.mission_control_stats();

    ApiResponse::ok(StatusResponse {
        version: crate::VERSION.to_string(),
        codename: crate::CODENAME.to_string(),
        genome_count,
        features: vec![
            "p53 protection (elephant/whale)".into(),
            "Telomere aging".into(),
            "Telomerase (immortality)".into(),
            "CRISPR editing".into(),
            "TTRL evolution".into(),
            "Meiosis reproduction".into(),
            "RSM-COIN economy ($88K/token)".into(),
            "Burn mechanism".into(),
            "Debt absorption ($350T target)".into(),
            "Multi-chain archivation".into(),
            "Lightning Network swarm".into(),
            "Mission Control pathfinding".into(),
            "T/G RNA coordination".into(),
        ],
        exchange_stats,
        mission_control: mc_stats,
    })
}

async fn list_genomes(State(state): State<AppState>) -> Json<ApiResponse<Vec<GenomeResponse>>> {
    match state.database.get_genomes(20, 0).await {
        Ok(genomes) => {
            let responses: Vec<GenomeResponse> = genomes.iter().map(|g| g.into()).collect();
            ApiResponse::ok(responses)
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

async fn create_genome(State(state): State<AppState>) -> Json<ApiResponse<GenomeResponse>> {
    let genome = GenomeBuilder::random().elephant_mode().build_storage();
    match state.database.store_genome(&genome).await {
        Ok(id) => {
            let mut stored = genome;
            stored.db_id = Some(id);
            let mut exchange = state.exchange.write().await;
            exchange.consciousness_reward(&format!("genome_{}", id), stored.consciousness);
            ApiResponse::ok((&stored).into())
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

async fn create_whale_genome(State(state): State<AppState>) -> Json<ApiResponse<GenomeResponse>> {
    let genome = GenomeBuilder::random().whale_mode().build_storage();
    match state.database.store_genome(&genome).await {
        Ok(id) => {
            let mut stored = genome;
            stored.db_id = Some(id);
            let mut exchange = state.exchange.write().await;
            exchange.consciousness_reward(&format!("whale_genome_{}", id), stored.consciousness);
            info!("🐋 WHALE genome created: #{} | p53: {}", id, stored.p53_copies);
            ApiResponse::ok((&stored).into())
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

#[derive(Deserialize)]
pub struct EvolveRequest { pub genome_id: i64 }

#[derive(Serialize)]
pub struct EvolveResponse {
    pub genome: GenomeResponse,
    pub evolution: EvolutionResult,
    pub burn_event: Option<BurnEvent>,
}

async fn evolve_genome(
    State(state): State<AppState>,
    Json(req): Json<EvolveRequest>,
) -> Json<ApiResponse<EvolveResponse>> {
    let genome = match state.database.load_genome(req.genome_id).await {
        Ok(g) => g,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    let c_before = genome.consciousness;
    let engine = state.rotation_engine.read().await;

    let (evolved, evolution_result) = match state.ttrl_engine.evolve_with_engine(genome, &engine).await {
        Ok(result) => result,
        Err(e) => {
            let mut exchange = state.exchange.write().await;
            if e.to_string().contains("Senescence") {
                exchange.burn_on_senescence(req.genome_id, c_before);
            } else if e.to_string().contains("p53") {
                exchange.burn_on_cancer(req.genome_id, c_before);
            }
            return ApiResponse::err(e.to_string());
        }
    };
    drop(engine);

    let burn_event = if !evolution_result.success {
        let mut exchange = state.exchange.write().await;
        exchange.burn_on_degradation(req.genome_id, c_before, evolved.consciousness)
    } else {
        None
    };

    match state.database.store_genome(&evolved).await {
        Ok(id) => {
            let mut stored = evolved;
            stored.db_id = Some(id);

            if evolution_result.success {
                let mut exchange = state.exchange.write().await;
                exchange.consciousness_reward(&format!("genome_{}", id), stored.consciousness);
            }

            ApiResponse::ok(EvolveResponse {
                genome: (&stored).into(),
                evolution: evolution_result,
                burn_event,
            })
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

#[derive(Deserialize)]
pub struct MeiosisRequest { pub parent1_id: i64, pub parent2_id: i64 }

#[derive(Serialize)]
pub struct MeiosisResponse {
    pub parent1: GenomeResponse,
    pub parent2: GenomeResponse,
    pub offspring: GenomeResponse,
    pub crossover_type: String,
}

async fn meiosis_genome(
    State(state): State<AppState>,
    Json(req): Json<MeiosisRequest>,
) -> Json<ApiResponse<MeiosisResponse>> {
    let parent1 = match state.database.load_genome(req.parent1_id).await {
        Ok(g) => g,
        Err(e) => return ApiResponse::err(format!("Parent 1: {}", e)),
    };

    let parent2 = match state.database.load_genome(req.parent2_id).await {
        Ok(g) => g,
        Err(e) => return ApiResponse::err(format!("Parent 2: {}", e)),
    };

    let mut exchange = state.exchange.write().await;
    exchange.meiosis_fee("breeder", parent1.consciousness, parent2.consciousness);
    drop(exchange);

    let offspring = state.ttrl_engine.meiosis(parent1.clone(), parent2.clone());

    match state.database.store_genome(&offspring).await {
        Ok(id) => {
            let mut stored = offspring;
            stored.db_id = Some(id);

            let mut exchange = state.exchange.write().await;
            exchange.consciousness_reward(&format!("genome_{}", id), stored.consciousness);

            ApiResponse::ok(MeiosisResponse {
                parent1: (&parent1).into(),
                parent2: (&parent2).into(),
                offspring: (&stored).into(),
                crossover_type: "positive_interference".to_string(),
            })
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

#[derive(Deserialize)]
pub struct TelomeraseRequest { pub genome_id: i64 }

#[derive(Serialize)]
pub struct TelomeraseResponse {
    pub genome: GenomeResponse,
    pub telomeres_before: u16,
    pub telomeres_after: u16,
    pub biological_age_before: f64,
    pub biological_age_after: f64,
    pub immortality_achieved: bool,
}

async fn activate_telomerase(
    State(state): State<AppState>,
    Json(req): Json<TelomeraseRequest>,
) -> Json<ApiResponse<TelomeraseResponse>> {
    let mut genome = match state.database.load_genome(req.genome_id).await {
        Ok(g) => g,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    let telomeres_before = genome.telomere_length;
    let age_before = genome.biological_age();

    genome.activate_telomerase();

    match state.database.store_genome(&genome).await {
        Ok(id) => {
            let mut stored = genome;
            stored.db_id = Some(id);

            info!("🧬 TELOMERASE: genome #{} | {} → {} bp | IMMORTAL!",
                  id, telomeres_before, stored.telomere_length);

            ApiResponse::ok(TelomeraseResponse {
                genome: (&stored).into(),
                telomeres_before,
                telomeres_after: stored.telomere_length,
                biological_age_before: age_before,
                biological_age_after: stored.biological_age(),
                immortality_achieved: stored.telomere_length == 15000,
            })
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// CRISPR handlers
#[derive(Deserialize)]
pub struct CrisprSpliceRequest { pub genome_id: i64, pub position: usize, pub new_base: char }

async fn crispr_splice(State(state): State<AppState>, Json(req): Json<CrisprSpliceRequest>) -> Json<ApiResponse<GenomeResponse>> {
    let mut genome = match state.database.load_genome(req.genome_id).await {
        Ok(g) => g,
        Err(e) => return ApiResponse::err(e.to_string()),
    };
    let tetrad = match Tetrad::from_char(req.new_base) {
        Some(t) => t,
        None => return ApiResponse::err("Invalid base".into()),
    };
    if req.position >= 27 { return ApiResponse::err("Position must be 0-26".into()); }
    genome.crispr_splice(req.position, tetrad);
    match state.database.store_genome(&genome).await {
        Ok(id) => { let mut s = genome; s.db_id = Some(id); ApiResponse::ok((&s).into()) }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

#[derive(Deserialize)]
pub struct CrisprJoinRequest { pub genome_id: i64, pub pos1: usize, pub pos2: usize }

async fn crispr_join(State(state): State<AppState>, Json(req): Json<CrisprJoinRequest>) -> Json<ApiResponse<GenomeResponse>> {
    let mut genome = match state.database.load_genome(req.genome_id).await {
        Ok(g) => g,
        Err(e) => return ApiResponse::err(e.to_string()),
    };
    if req.pos1 >= 27 || req.pos2 >= 27 { return ApiResponse::err("Positions must be 0-26".into()); }
    genome.crispr_join(req.pos1, req.pos2);
    match state.database.store_genome(&genome).await {
        Ok(id) => { let mut s = genome; s.db_id = Some(id); ApiResponse::ok((&s).into()) }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

#[derive(Deserialize)]
pub struct CrisprDeleteRequest { pub genome_id: i64, pub position: usize }

async fn crispr_delete(State(state): State<AppState>, Json(req): Json<CrisprDeleteRequest>) -> Json<ApiResponse<GenomeResponse>> {
    let mut genome = match state.database.load_genome(req.genome_id).await {
        Ok(g) => g,
        Err(e) => return ApiResponse::err(e.to_string()),
    };
    if req.position >= 27 { return ApiResponse::err("Position must be 0-26".into()); }
    genome.crispr_delete(req.position);
    match state.database.store_genome(&genome).await {
        Ok(id) => { let mut s = genome; s.db_id = Some(id); ApiResponse::ok((&s).into()) }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// RSM handlers
async fn rsm_stats(State(state): State<AppState>) -> Json<ApiResponse<ExchangeStats>> {
    ApiResponse::ok(state.exchange.read().await.stats())
}

#[derive(Deserialize)]
pub struct BuyRequest { pub wallet: String, pub usd_amount: f64, pub consciousness: u32 }

async fn rsm_buy(State(state): State<AppState>, Json(req): Json<BuyRequest>) -> Json<ApiResponse<Transaction>> {
    let mut exchange = state.exchange.write().await;
    let tx = exchange.buy_rsm(&req.wallet, req.usd_amount, req.consciousness);
    ApiResponse::ok(tx)
}

#[derive(Deserialize)]
pub struct SellRequest { pub wallet: String, pub rsm_amount: f64, pub consciousness: u32 }

async fn rsm_sell(State(state): State<AppState>, Json(req): Json<SellRequest>) -> Json<ApiResponse<Transaction>> {
    let mut exchange = state.exchange.write().await;
    match exchange.sell_rsm(&req.wallet, req.rsm_amount, req.consciousness) {
        Some(tx) => ApiResponse::ok(tx),
        None => ApiResponse::err("Insufficient balance".into()),
    }
}

#[derive(Deserialize)]
pub struct TransferRequest { pub from_wallet: String, pub to_wallet: String, pub amount: f64 }

async fn rsm_transfer(State(state): State<AppState>, Json(req): Json<TransferRequest>) -> Json<ApiResponse<Transaction>> {
    let mut exchange = state.exchange.write().await;
    match exchange.transfer(&req.from_wallet, &req.to_wallet, req.amount) {
        Some(tx) => ApiResponse::ok(tx),
        None => ApiResponse::err("Insufficient balance".into()),
    }
}

#[derive(Deserialize)]
pub struct RewardRequest { pub wallet: String, pub consciousness: u32 }

async fn rsm_reward(State(state): State<AppState>, Json(req): Json<RewardRequest>) -> Json<ApiResponse<Transaction>> {
    let mut exchange = state.exchange.write().await;
    let tx = exchange.consciousness_reward(&req.wallet, req.consciousness);
    ApiResponse::ok(tx)
}

#[derive(Deserialize)]
pub struct ManualBurnRequest { pub amount: f64 }

async fn rsm_manual_burn(State(state): State<AppState>, Json(req): Json<ManualBurnRequest>) -> Json<ApiResponse<BurnEvent>> {
    let mut exchange = state.exchange.write().await;
    let event = exchange.burn(req.amount, BurnReason::ManualBurn, None, 0, 0);
    ApiResponse::ok(event)
}

async fn list_transactions(State(state): State<AppState>) -> Json<ApiResponse<Vec<Transaction>>> {
    let exchange = state.exchange.read().await;
    ApiResponse::ok(exchange.recent_transactions(50))
}

async fn list_burns(State(state): State<AppState>) -> Json<ApiResponse<Vec<BurnEvent>>> {
    let exchange = state.exchange.read().await;
    ApiResponse::ok(exchange.recent_burns(50))
}

async fn debt_stats(State(state): State<AppState>) -> Json<ApiResponse<DebtStats>> {
    ApiResponse::ok(state.exchange.read().await.debt_stats())
}

async fn owner_pool(State(state): State<AppState>) -> Json<ApiResponse<OwnerPoolStats>> {
    ApiResponse::ok(state.exchange.read().await.owner_pool())
}

// Multi-chain & LN handlers
#[derive(Deserialize)]
pub struct ArchiveRequest { pub genome_id: i64 }

async fn archive_genome(State(state): State<AppState>, Json(req): Json<ArchiveRequest>) -> Json<ApiResponse<ChainArchiveEntry>> {
    let genome = match state.database.load_genome(req.genome_id).await {
        Ok(g) => g,
        Err(e) => return ApiResponse::err(e.to_string()),
    };
    let mut archiver = state.archiver.write().await;
    match archiver.archive(&genome).await {
        Ok(entry) => ApiResponse::ok(entry),
        Err(e) => ApiResponse::err(e),
    }
}

async fn list_archives(State(state): State<AppState>) -> Json<ApiResponse<Vec<ChainArchiveEntry>>> {
    let archiver = state.archiver.read().await;
    let archives: Vec<ChainArchiveEntry> = archiver.recent_archives(50).into_iter().cloned().collect();
    ApiResponse::ok(archives)
}

async fn mission_control_stats(State(state): State<AppState>) -> Json<ApiResponse<MissionControlStats>> {
    let archiver = state.archiver.read().await;
    ApiResponse::ok(archiver.mission_control_stats())
}

async fn reset_mission_control(State(state): State<AppState>) -> Json<ApiResponse<String>> {
    let mut archiver = state.archiver.write().await;
    archiver.mission_control.reset();
    ApiResponse::ok("Mission Control reset".to_string())
}

// Rotation handlers
async fn rotation_stats(State(state): State<AppState>) -> Json<ApiResponse<RotationStats>> {
    let engine = state.rotation_engine.read().await;
    ApiResponse::ok(engine.get_stats())
}

async fn manual_rotate(State(state): State<AppState>) -> Json<ApiResponse<RotationStats>> {
    let mut engine = state.rotation_engine.write().await;
    engine.rotate();
    ApiResponse::ok(engine.get_stats())
}

// ═══════════════════════════════════════════════════════════════
// AUTH & WALLET HANDLERS
// ═══════════════════════════════════════════════════════════════

const FOUNDER_POOL_RSM: f64 = 1_428_571_428_571_428.0; // 1/7 of 10 quadrillion

#[derive(Deserialize)]
pub struct TokenRequest {
    pub token: String,
}

#[derive(Deserialize)]
pub struct DepositRequest {
    pub token: String,
    pub amount_rsm: f64,
}

#[derive(Deserialize)]
pub struct WithdrawRequest {
    pub token: String,
    pub amount_rsm: f64,
}

async fn auth_register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Json<ApiResponse<LoginResponse>> {
    // Check if username already exists
    if let Ok(Some(_)) = state.database.get_wallet_by_username(&req.username).await {
        return ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            wallet_address: None,
            username: None,
            rsm_balance: None,
            founder_pool_rsm: None,
            is_founder: None,
            expires_at: None,
            message: "Username already exists".to_string(),
        });
    }

    // Generate credentials
    let salt = AuthManager::generate_salt();
    let password_hash = AuthManager::hash_password(&req.password, &salt);
    let wallet_address = AuthManager::generate_wallet_address(&req.username);
    
    // Founder gets the pool!
    let founder_pool = if req.is_founder { FOUNDER_POOL_RSM } else { 0.0 };

    // Create account in DB
    match state.database.create_wallet_account(
        &req.username,
        &password_hash,
        &salt,
        &wallet_address,
        req.is_founder,
        founder_pool,
    ).await {
        Ok(_) => {
            // Auto-login after registration
            let mut auth = state.auth.write().await;
            let session = auth.generate_token(&wallet_address, &req.username);
            
            info!("📝 New wallet registered: {} | {} | Founder: {}", 
                  req.username, wallet_address, req.is_founder);
            
            ApiResponse::ok(LoginResponse {
                success: true,
                token: Some(session.token),
                wallet_address: Some(wallet_address),
                username: Some(req.username),
                rsm_balance: Some(0.0),
                founder_pool_rsm: Some(founder_pool),
                is_founder: Some(req.is_founder),
                expires_at: Some(session.expires_at),
                message: if req.is_founder {
                    format!("🐋 FOUNDER wallet created! You have {} RSM in founder pool!", founder_pool)
                } else {
                    "Wallet created successfully!".to_string()
                },
            })
        }
        Err(e) => ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            wallet_address: None,
            username: None,
            rsm_balance: None,
            founder_pool_rsm: None,
            is_founder: None,
            expires_at: None,
            message: format!("Registration failed: {}", e),
        }),
    }
}

async fn auth_login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Json<ApiResponse<LoginResponse>> {
    // Get account
    let account = match state.database.get_wallet_by_username(&req.username).await {
        Ok(Some(acc)) => acc,
        Ok(None) => return ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            wallet_address: None,
            username: None,
            rsm_balance: None,
            founder_pool_rsm: None,
            is_founder: None,
            expires_at: None,
            message: "Invalid username or password".to_string(),
        }),
        Err(e) => return ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            wallet_address: None,
            username: None,
            rsm_balance: None,
            founder_pool_rsm: None,
            is_founder: None,
            expires_at: None,
            message: format!("Login error: {}", e),
        }),
    };

    // Verify password
    if !AuthManager::verify_password(&req.password, &account.salt, &account.password_hash) {
        return ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            wallet_address: None,
            username: None,
            rsm_balance: None,
            founder_pool_rsm: None,
            is_founder: None,
            expires_at: None,
            message: "Invalid username or password".to_string(),
        });
    }

    // Update last login
    let _ = state.database.update_last_login(&req.username).await;

    // Generate session
    let mut auth = state.auth.write().await;
    let session = auth.generate_token(&account.wallet_address, &req.username);
    
    info!("🔐 Login: {} | Balance: {} RSM", req.username, account.rsm_balance);

    ApiResponse::ok(LoginResponse {
        success: true,
        token: Some(session.token),
        wallet_address: Some(account.wallet_address),
        username: Some(account.username),
        rsm_balance: Some(account.rsm_balance),
        founder_pool_rsm: Some(account.founder_pool_rsm),
        is_founder: Some(account.is_founder),
        expires_at: Some(session.expires_at),
        message: "Login successful".to_string(),
    })
}

async fn auth_logout(
    State(state): State<AppState>,
    Json(req): Json<TokenRequest>,
) -> Json<ApiResponse<String>> {
    let mut auth = state.auth.write().await;
    if auth.logout(&req.token) {
        ApiResponse::ok("Logged out successfully".to_string())
    } else {
        ApiResponse::err("Invalid token".to_string())
    }
}

async fn auth_profile(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<WalletInfo>> {
    let token = match params.get("token") {
        Some(t) => t,
        None => return ApiResponse::err("Token required".to_string()),
    };

    let auth = state.auth.read().await;
    let session = match auth.validate_token(token) {
        Some(s) => s.clone(),
        None => return ApiResponse::err("Invalid or expired token".to_string()),
    };
    drop(auth);

    let account = match state.database.get_wallet_by_address(&session.wallet_address).await {
        Ok(Some(acc)) => acc,
        _ => return ApiResponse::err("Wallet not found".to_string()),
    };

    let total_value = (account.rsm_balance + account.founder_pool_rsm) * 88000.0;

    ApiResponse::ok(WalletInfo {
        username: account.username,
        wallet_address: account.wallet_address,
        rsm_balance: account.rsm_balance,
        founder_pool_rsm: account.founder_pool_rsm,
        is_founder: account.is_founder,
        created_at: account.created_at,
        last_login: account.last_login,
        total_value_usd: total_value,
    })
}

async fn wallet_info(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<WalletInfo>> {
    // Same as profile but with wallet address
    let address = match params.get("address") {
        Some(a) => a,
        None => return ApiResponse::err("Wallet address required".to_string()),
    };

    let account = match state.database.get_wallet_by_address(address).await {
        Ok(Some(acc)) => acc,
        Ok(None) => return ApiResponse::err("Wallet not found".to_string()),
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    let total_value = (account.rsm_balance + account.founder_pool_rsm) * 88000.0;

    ApiResponse::ok(WalletInfo {
        username: account.username,
        wallet_address: account.wallet_address,
        rsm_balance: account.rsm_balance,
        founder_pool_rsm: account.founder_pool_rsm,
        is_founder: account.is_founder,
        created_at: account.created_at,
        last_login: account.last_login,
        total_value_usd: total_value,
    })
}

async fn wallet_deposit(
    State(state): State<AppState>,
    Json(req): Json<DepositRequest>,
) -> Json<ApiResponse<WalletInfo>> {
    // Validate token
    let auth = state.auth.read().await;
    let session = match auth.validate_token(&req.token) {
        Some(s) => s.clone(),
        None => return ApiResponse::err("Invalid or expired token".to_string()),
    };
    drop(auth);

    // Get current balance
    let account = match state.database.get_wallet_by_address(&session.wallet_address).await {
        Ok(Some(acc)) => acc,
        _ => return ApiResponse::err("Wallet not found".to_string()),
    };

    // Update balance
    let new_balance = account.rsm_balance + req.amount_rsm;
    if let Err(e) = state.database.update_wallet_balance(&session.wallet_address, new_balance).await {
        return ApiResponse::err(e.to_string());
    }

    info!("💰 Deposit: {} | +{} RSM | New balance: {}", 
          session.username, req.amount_rsm, new_balance);

    let total_value = (new_balance + account.founder_pool_rsm) * 88000.0;

    ApiResponse::ok(WalletInfo {
        username: account.username,
        wallet_address: account.wallet_address,
        rsm_balance: new_balance,
        founder_pool_rsm: account.founder_pool_rsm,
        is_founder: account.is_founder,
        created_at: account.created_at,
        last_login: account.last_login,
        total_value_usd: total_value,
    })
}

async fn wallet_withdraw(
    State(state): State<AppState>,
    Json(req): Json<WithdrawRequest>,
) -> Json<ApiResponse<WalletInfo>> {
    // Validate token
    let auth = state.auth.read().await;
    let session = match auth.validate_token(&req.token) {
        Some(s) => s.clone(),
        None => return ApiResponse::err("Invalid or expired token".to_string()),
    };
    drop(auth);

    // Get current balance
    let account = match state.database.get_wallet_by_address(&session.wallet_address).await {
        Ok(Some(acc)) => acc,
        _ => return ApiResponse::err("Wallet not found".to_string()),
    };

    // Check sufficient balance
    if account.rsm_balance < req.amount_rsm {
        return ApiResponse::err(format!(
            "Insufficient balance: {} RSM available, {} requested",
            account.rsm_balance, req.amount_rsm
        ));
    }

    // Update balance
    let new_balance = account.rsm_balance - req.amount_rsm;
    if let Err(e) = state.database.update_wallet_balance(&session.wallet_address, new_balance).await {
        return ApiResponse::err(e.to_string());
    }

    info!("💸 Withdraw: {} | -{} RSM | New balance: {}", 
          session.username, req.amount_rsm, new_balance);

    let total_value = (new_balance + account.founder_pool_rsm) * 88000.0;

    ApiResponse::ok(WalletInfo {
        username: account.username,
        wallet_address: account.wallet_address,
        rsm_balance: new_balance,
        founder_pool_rsm: account.founder_pool_rsm,
        is_founder: account.is_founder,
        created_at: account.created_at,
        last_login: account.last_login,
        total_value_usd: total_value,
    })
}

async fn wallet_list(State(state): State<AppState>) -> Json<ApiResponse<Vec<WalletInfo>>> {
    match state.database.get_all_wallets().await {
        Ok(wallets) => {
            let infos: Vec<WalletInfo> = wallets.into_iter().map(|acc| {
                let total_value = (acc.rsm_balance + acc.founder_pool_rsm) * 88000.0;
                WalletInfo {
                    username: acc.username,
                    wallet_address: acc.wallet_address,
                    rsm_balance: acc.rsm_balance,
                    founder_pool_rsm: acc.founder_pool_rsm,
                    is_founder: acc.is_founder,
                    created_at: acc.created_at,
                    last_login: acc.last_login,
                    total_value_usd: total_value,
                }
            }).collect();
            ApiResponse::ok(infos)
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}
