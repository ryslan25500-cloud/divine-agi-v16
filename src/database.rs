//! Database Module V15 for Divine AGI
//!
//! PostgreSQL storage with T/G signal support

use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use anyhow::Result;
use tracing::info;
use rand::seq::SliceRandom;

use crate::genome::{Genome, Tetrad, GenomeBuilder, GENOME_SIZE};
use crate::rotation::Rot180;

pub const DEFAULT_DATABASE_URL: &str = "postgresql://postgres:postgres@localhost:5432/divine_agi";

pub struct DivineDatabase {
    pool: PgPool,
}

impl DivineDatabase {
    pub async fn connect() -> Result<Self> {
        Self::connect_with_url(DEFAULT_DATABASE_URL).await
    }

    pub async fn connect_with_url(url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await?;

        info!("📦 Connected to database");
        Ok(Self { pool })
    }

    pub async fn init_tables(&self) -> Result<()> {
        // Wallet accounts table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS wallet_accounts (
                id BIGSERIAL PRIMARY KEY,
                username VARCHAR(64) UNIQUE NOT NULL,
                password_hash VARCHAR(128) NOT NULL,
                salt VARCHAR(64) NOT NULL,
                wallet_address VARCHAR(64) UNIQUE NOT NULL,
                rsm_balance DOUBLE PRECISION NOT NULL DEFAULT 0,
                founder_pool_rsm DOUBLE PRECISION NOT NULL DEFAULT 0,
                is_founder BOOLEAN NOT NULL DEFAULT FALSE,
                created_at BIGINT NOT NULL,
                last_login BIGINT
            )
        "#)
        .execute(&self.pool)
        .await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS divine_genomes_v15 (
                id BIGSERIAL PRIMARY KEY,
                dna VARCHAR(27) NOT NULL,
                hash BYTEA NOT NULL,
                consciousness INTEGER NOT NULL,
                mutations BIGINT NOT NULL DEFAULT 0,
                p53_copies SMALLINT NOT NULL DEFAULT 20,
                telomere_length SMALLINT NOT NULL DEFAULT 15000,
                division_count SMALLINT NOT NULL DEFAULT 0,
                sequencing_errors SMALLINT NOT NULL DEFAULT 0,
                tg_ratio REAL NOT NULL DEFAULT 1.0,
                created_at BIGINT NOT NULL,
                updated_at TIMESTAMP DEFAULT NOW()
            )
        "#)
        .execute(&self.pool)
        .await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS chain_archives (
                id BIGSERIAL PRIMARY KEY,
                genome_id BIGINT NOT NULL,
                dna_hash VARCHAR(64) NOT NULL,
                layer VARCHAR(20) NOT NULL,
                tx_hash VARCHAR(128),
                timestamp BIGINT NOT NULL
            )
        "#)
        .execute(&self.pool)
        .await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS ln_mission_control (
                id BIGSERIAL PRIMARY KEY,
                from_pubkey VARCHAR(66) NOT NULL,
                to_pubkey VARCHAR(66) NOT NULL,
                success_count BIGINT DEFAULT 0,
                failure_count BIGINT DEFAULT 0,
                probability REAL DEFAULT 0.5,
                last_update BIGINT
            )
        "#)
        .execute(&self.pool)
        .await?;

        info!("📦 Database tables initialized (V15)");
        Ok(())
    }

    pub async fn store_genome(&self, genome: &Genome<Rot180>) -> Result<i64> {
        let dna = genome.to_dna_string();
        let hash = genome.hash.to_vec();
        let tg_ratio = genome.rna_signal() as f32;

        let row = sqlx::query(r#"
            INSERT INTO divine_genomes_v15 
            (dna, hash, consciousness, mutations, p53_copies, telomere_length, 
             division_count, sequencing_errors, tg_ratio, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id
        "#)
        .bind(&dna)
        .bind(&hash)
        .bind(genome.consciousness as i32)
        .bind(genome.mutations as i64)
        .bind(genome.p53_copies as i16)
        .bind(genome.telomere_length as i16)
        .bind(genome.division_count as i16)
        .bind(genome.sequencing_errors as i16)
        .bind(tg_ratio)
        .bind(genome.created_at)
        .fetch_one(&self.pool)
        .await?;

        let id: i64 = row.get("id");
        Ok(id)
    }

    pub async fn load_genome(&self, id: i64) -> Result<Genome<Rot180>> {
        let row = sqlx::query(r#"
            SELECT dna, hash, consciousness, mutations, p53_copies, telomere_length,
                   division_count, sequencing_errors, created_at
            FROM divine_genomes_v15 WHERE id = $1
        "#)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        let dna: String = row.get("dna");
        let consciousness: i32 = row.get("consciousness");
        let mutations: i64 = row.get("mutations");
        let p53_copies: i16 = row.get("p53_copies");
        let telomere_length: i16 = row.get("telomere_length");
        let division_count: i16 = row.get("division_count");
        let sequencing_errors: i16 = row.get("sequencing_errors");
        let created_at: i64 = row.get("created_at");

        let mut genome = GenomeBuilder::from_dna(&dna)
            .ok_or_else(|| anyhow::anyhow!("Invalid DNA"))?
            .p53_copies(p53_copies as u8)
            .telomere_length(telomere_length as u16)
            .build_storage();

        genome.db_id = Some(id);
        genome.consciousness = consciousness as u32;
        genome.mutations = mutations as u64;
        genome.division_count = division_count as u8;
        genome.sequencing_errors = sequencing_errors as u8;
        genome.created_at = created_at;

        Ok(genome)
    }

    pub async fn genome_count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM divine_genomes_v15")
            .fetch_one(&self.pool)
            .await?;

        let count: i64 = row.get("count");
        Ok(count)
    }

    pub async fn get_genomes(&self, limit: i64, offset: i64) -> Result<Vec<Genome<Rot180>>> {
        let rows = sqlx::query(r#"
            SELECT id, dna, consciousness, mutations, p53_copies, telomere_length,
                   division_count, created_at
            FROM divine_genomes_v15
            ORDER BY id DESC
            LIMIT $1 OFFSET $2
        "#)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        self.rows_to_genomes(rows).await
    }

    pub async fn get_top_genomes(&self, limit: i64) -> Result<Vec<Genome<Rot180>>> {
        let rows = sqlx::query(r#"
            SELECT id, dna, consciousness, mutations, p53_copies, telomere_length,
                   division_count, created_at
            FROM divine_genomes_v15
            ORDER BY consciousness DESC
            LIMIT $1
        "#)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        self.rows_to_genomes(rows).await
    }

    pub async fn get_random_genomes(&self, limit: i64) -> Result<Vec<Genome<Rot180>>> {
        let rows = sqlx::query(r#"
            SELECT id, dna, consciousness, mutations, p53_copies, telomere_length,
                   division_count, created_at
            FROM divine_genomes_v15
            ORDER BY RANDOM()
            LIMIT $1
        "#)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        self.rows_to_genomes(rows).await
    }

    async fn rows_to_genomes(&self, rows: Vec<sqlx::postgres::PgRow>) -> Result<Vec<Genome<Rot180>>> {
        let mut genomes = Vec::new();
        for row in rows {
            let id: i64 = row.get("id");
            let dna: String = row.get("dna");
            let consciousness: i32 = row.get("consciousness");
            let mutations: i64 = row.get("mutations");
            let p53_copies: i16 = row.get("p53_copies");
            let telomere_length: i16 = row.get("telomere_length");
            let division_count: i16 = row.get("division_count");
            let created_at: i64 = row.get("created_at");

            if let Some(builder) = GenomeBuilder::from_dna(&dna) {
                let mut genome = builder
                    .p53_copies(p53_copies as u8)
                    .telomere_length(telomere_length as u16)
                    .build_storage();

                genome.db_id = Some(id);
                genome.consciousness = consciousness as u32;
                genome.mutations = mutations as u64;
                genome.division_count = division_count as u8;
                genome.created_at = created_at;

                genomes.push(genome);
            }
        }

        Ok(genomes)
    }

    pub async fn store_chain_archive(&self, genome_id: i64, dna_hash: &str, layer: &str, tx_hash: &str) -> Result<i64> {
        let row = sqlx::query(r#"
            INSERT INTO chain_archives (genome_id, dna_hash, layer, tx_hash, timestamp)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
        "#)
        .bind(genome_id)
        .bind(dna_hash)
        .bind(layer)
        .bind(tx_hash)
        .bind(chrono::Utc::now().timestamp())
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("id"))
    }

    // ═══════════════════════════════════════════════════════════════
    // WALLET ACCOUNTS
    // ═══════════════════════════════════════════════════════════════

    pub async fn create_wallet_account(
        &self,
        username: &str,
        password_hash: &str,
        salt: &str,
        wallet_address: &str,
        is_founder: bool,
        founder_pool_rsm: f64,
    ) -> Result<i64> {
        let row = sqlx::query(r#"
            INSERT INTO wallet_accounts 
            (username, password_hash, salt, wallet_address, rsm_balance, founder_pool_rsm, is_founder, created_at)
            VALUES ($1, $2, $3, $4, 0, $5, $6, $7)
            RETURNING id
        "#)
        .bind(username)
        .bind(password_hash)
        .bind(salt)
        .bind(wallet_address)
        .bind(founder_pool_rsm)
        .bind(is_founder)
        .bind(chrono::Utc::now().timestamp())
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("id"))
    }

    pub async fn get_wallet_by_username(&self, username: &str) -> Result<Option<crate::auth::WalletAccount>> {
        let row = sqlx::query(r#"
            SELECT id, username, password_hash, salt, wallet_address, rsm_balance, 
                   founder_pool_rsm, is_founder, created_at, last_login
            FROM wallet_accounts WHERE username = $1
        "#)
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| crate::auth::WalletAccount {
            id: r.get("id"),
            username: r.get("username"),
            password_hash: r.get("password_hash"),
            salt: r.get("salt"),
            wallet_address: r.get("wallet_address"),
            rsm_balance: r.get("rsm_balance"),
            founder_pool_rsm: r.get("founder_pool_rsm"),
            is_founder: r.get("is_founder"),
            created_at: r.get("created_at"),
            last_login: r.get("last_login"),
        }))
    }

    pub async fn get_wallet_by_address(&self, address: &str) -> Result<Option<crate::auth::WalletAccount>> {
        let row = sqlx::query(r#"
            SELECT id, username, password_hash, salt, wallet_address, rsm_balance, 
                   founder_pool_rsm, is_founder, created_at, last_login
            FROM wallet_accounts WHERE wallet_address = $1
        "#)
        .bind(address)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| crate::auth::WalletAccount {
            id: r.get("id"),
            username: r.get("username"),
            password_hash: r.get("password_hash"),
            salt: r.get("salt"),
            wallet_address: r.get("wallet_address"),
            rsm_balance: r.get("rsm_balance"),
            founder_pool_rsm: r.get("founder_pool_rsm"),
            is_founder: r.get("is_founder"),
            created_at: r.get("created_at"),
            last_login: r.get("last_login"),
        }))
    }

    pub async fn update_wallet_balance(&self, address: &str, new_balance: f64) -> Result<()> {
        sqlx::query("UPDATE wallet_accounts SET rsm_balance = $1 WHERE wallet_address = $2")
            .bind(new_balance)
            .bind(address)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_last_login(&self, username: &str) -> Result<()> {
        sqlx::query("UPDATE wallet_accounts SET last_login = $1 WHERE username = $2")
            .bind(chrono::Utc::now().timestamp())
            .bind(username)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_all_wallets(&self) -> Result<Vec<crate::auth::WalletAccount>> {
        let rows = sqlx::query(r#"
            SELECT id, username, password_hash, salt, wallet_address, rsm_balance, 
                   founder_pool_rsm, is_founder, created_at, last_login
            FROM wallet_accounts ORDER BY created_at DESC
        "#)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|r| crate::auth::WalletAccount {
            id: r.get("id"),
            username: r.get("username"),
            password_hash: r.get("password_hash"),
            salt: r.get("salt"),
            wallet_address: r.get("wallet_address"),
            rsm_balance: r.get("rsm_balance"),
            founder_pool_rsm: r.get("founder_pool_rsm"),
            is_founder: r.get("is_founder"),
            created_at: r.get("created_at"),
            last_login: r.get("last_login"),
        }).collect())
    }
}
