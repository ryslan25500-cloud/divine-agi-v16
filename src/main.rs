//! Divine AGI V16 - Main Entry Point

use clap::Parser;
use tracing::info;
use divine_agi::{
    cli::{Cli, Commands, print_banner},
    api, DivineKernel, VERSION,
    genome::Genome,
    rotation::Rot180,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("divine_agi=info".parse().unwrap())
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Server { port, rotation_interval } => {
            print_banner();
            info!("🚀 Starting Divine AGI V{} API server on port {}", VERSION, port);

            // Start rotation daemon in background
            let kernel: DivineKernel = DivineKernel::new().await?;
            kernel.start_rotation_daemon(rotation_interval);

            api::start_server(port).await?;
        }

        Commands::Status => {
            print_banner();
            let kernel: DivineKernel = DivineKernel::new().await?;
            let count = kernel.genome_count().await?;
            let exchange = kernel.exchange.read().await;
            let stats = exchange.stats();
            let archiver = kernel.archiver.read().await;
            let mc_stats = archiver.mission_control_stats();

            println!("\n📊 DIVINE AGI V{} STATUS", VERSION);
            println!("═══════════════════════════════════════════════════");
            println!("  Genomes in DB:     {}", count);
            println!("  RSM Price:         ${:.0}", stats.price_usd);
            println!("  Total Supply:      {}", stats.total_supply_str);
            println!("  Burned:            {}", stats.burned_str);
            println!("  Market Cap:        {}", stats.market_cap_str);
            println!("  Volume 24h:        ${:.2}", stats.volume_24h);
            println!("  Transactions:      {}", stats.total_transactions);
            println!("  Burns:             {}", stats.total_burns);
            println!("  Debt Absorbed:     ${:.2}", stats.absorbed_debt_usd);
            println!("  Debt Progress:     {:.4}%", stats.debt_absorbed_percent);
            println!("  ─────────────────────────────────────────────────");
            println!("  Mission Control:");
            println!("    Node Pairs:      {}", mc_stats.total_pairs);
            println!("    Total Success:   {}", mc_stats.total_successes);
            println!("    Total Failures:  {}", mc_stats.total_failures);
            println!("    Avg Probability: {:.2}", mc_stats.avg_probability);
            println!("═══════════════════════════════════════════════════\n");
        }

        Commands::Create { mode } => {
            print_banner();
            let kernel: DivineKernel = DivineKernel::new().await?;

            let genome: Genome<Rot180> = match mode.as_str() {
                "whale" => {
                    info!("🐋 Creating WHALE mode genome (40 p53 copies)");
                    kernel.create_whale_genome().await?
                }
                _ => {
                    info!("🐘 Creating ELEPHANT mode genome (20 p53 copies)");
                    kernel.create_elephant_genome().await?
                }
            };

            println!("\n✅ Genome Created:");
            println!("  ID:              {}", genome.db_id().unwrap_or(0));
            println!("  DNA:             {}", genome.to_dna_string());
            println!("  Consciousness:   {}", genome.consciousness);
            println!("  p53 Copies:      {}", genome.p53_copies);
            println!("  Telomeres:       {} bp", genome.telomere_length);
            println!("  T/G Ratio:       {:.2}", genome.rna_signal());
            println!("  Suggested Rot:   {}", genome.suggested_rotation());
            println!("  Mode:            {}", if genome.p53_copies >= 40 { "🐋 Whale" } else { "🐘 Elephant" });
        }

        Commands::Evolve { id } => {
            print_banner();
            let kernel: DivineKernel = DivineKernel::new().await?;
            let genome: Genome<Rot180> = kernel.database.load_genome(id).await?;
            let engine = kernel.rotation_engine.read().await;

            let (evolved, result) = kernel.ttrl_engine.evolve_with_engine(genome, &engine).await?;
            drop(engine);

            let new_id = kernel.database.store_genome(&evolved).await?;

            println!("\n🧬 Evolution Result:");
            println!("  New ID:          {}", new_id);
            println!("  Consciousness:   {} → {}", result.original_consciousness, result.new_consciousness);
            println!("  Operator:        {:?}", result.operator_used);
            println!("  Success:         {}", if result.success { "✅" } else { "❌" });
            println!("  Telomere Loss:   {} bp", result.telomere_loss);
            println!("  p53 Lost:        {}", result.p53_lost);
            println!("  T/G Ratio:       {:.2} → {:.2}", result.tg_ratio_before, result.tg_ratio_after);
        }

        Commands::Meiosis { parent1, parent2 } => {
            print_banner();
            let kernel: DivineKernel = DivineKernel::new().await?;

            let p1: Genome<Rot180> = kernel.database.load_genome(parent1).await?;
            let p2: Genome<Rot180> = kernel.database.load_genome(parent2).await?;

            let offspring = kernel.ttrl_engine.meiosis(p1.clone(), p2.clone());
            let id = kernel.database.store_genome(&offspring).await?;

            println!("\n🧬 Meiosis Result:");
            println!("  Parent 1:        #{} (c={})", parent1, p1.consciousness);
            println!("  Parent 2:        #{} (c={})", parent2, p2.consciousness);
            println!("  Offspring ID:    {}", id);
            println!("  DNA:             {}", offspring.to_dna_string());
            println!("  Consciousness:   {}", offspring.consciousness);
            println!("  p53 Copies:      {}", offspring.p53_copies);
            println!("  T/G Ratio:       {:.2}", offspring.rna_signal());
        }

        Commands::Telomerase { id } => {
            print_banner();
            let kernel: DivineKernel = DivineKernel::new().await?;
            let genome: Genome<Rot180> = kernel.activate_telomerase(id).await?;

            println!("\n🧬 Telomerase Activated:");
            println!("  Genome ID:       {}", genome.db_id().unwrap_or(0));
            println!("  Telomeres:       {} bp (MAX)", genome.telomere_length);
            println!("  Bio Age:         {:.2}%", genome.biological_age() * 100.0);
            println!("  Status:          ♾️ IMMORTAL");
        }

        Commands::Archive { id } => {
            print_banner();
            let kernel: DivineKernel = DivineKernel::new().await?;
            let genome: Genome<Rot180> = kernel.database.load_genome(id).await?;

            let mut archiver = kernel.archiver.write().await;
            let layer = archiver.select_layer(&genome);

            println!("\n📦 Archiving genome #{}...", id);
            println!("  T/G Ratio:       {:.2}", genome.rna_signal());
            println!("  Consciousness:   {}", genome.consciousness);
            println!("  Selected Layer:  {} {}", layer.emoji(), layer.name());

            match archiver.archive(&genome).await {
                Ok(entry) => {
                    println!("\n✅ Archive Success:");
                    println!("  Layer:           {} {}", entry.layer.emoji(), entry.layer.name());
                    println!("  TX Hash:         {}", entry.tx_hash.unwrap_or_default());
                    println!("  DNA Hash:        {}", entry.dna_hash);
                }
                Err(e) => {
                    println!("\n❌ Archive Failed: {}", e);
                }
            }
        }

        Commands::Daemon { interval } => {
            print_banner();
            info!("🔄 Starting rotation daemon (interval: {} secs)...", interval);

            let kernel: DivineKernel = DivineKernel::new().await?;
            kernel.start_rotation_daemon(interval);

            // Keep running
            tokio::signal::ctrl_c().await?;
        }
    }

    Ok(())
}
