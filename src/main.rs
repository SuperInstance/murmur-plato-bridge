use chrono::Utc;
use murmur_plato_bridge::{
    bidir_sync::BidirSync,
    conflict_detector::hash_thought,
    quality_gate::passes_gate,
    tile_mapper::thought_to_tile,
    tensor_parser::parse_tensor,
    BridgeConfig, BridgeState, PlatoWriter, Thought, Strategy,
};
use std::process;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

const STATE_FILE: &str = "bridge_state.json";

fn init_logging() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_target(true)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
}

fn synthetic_tensor() -> Vec<Thought> {
    vec![
        Thought {
            content: "Tensor products provide a universal construction for bilinear maps across mathematical domains, enabling composition of morphisms in monoidal categories.".to_string(),
            connections: vec!["theorem_universal_property".to_string()],
            questions: vec![],
            confidence: 0.92,
            strategy: Strategy::Explore,
            timestamp: Utc::now(),
        },
        Thought {
            content: "The tension between classical set-theoretic foundations and category-theoretic abstraction reveals deep epistemological differences in mathematical practice, particularly in how infinity is handled.".to_string(),
            connections: vec!["theorem_cantor".to_string(), "theorem_klein_erlangen".to_string()],
            questions: vec!["How do we reconcile these foundational frameworks?".to_string()],
            confidence: 0.78,
            strategy: Strategy::Contradict,
            timestamp: Utc::now(),
        },
        Thought {
            content: "What if adjunction and coherence conditions are two perspectives on the same underlying mathematical reality, linked by the density of representable functors?".to_string(),
            connections: vec!["theorem_yoneda".to_string()],
            questions: vec!["What deeper structure connects these concepts?".to_string()],
            confidence: 0.85,
            strategy: Strategy::Question,
            timestamp: Utc::now(),
        },
    ]
}

async fn submit_thought(writer: &PlatoWriter, thought: &Thought, hash: &str, state: &mut BridgeState) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let config = BridgeConfig::default();
    
    // Quality gate
    if !passes_gate(thought, &config) {
        info!("Thought filtered by quality gate (confidence: {:.2})", thought.confidence);
        return Ok(false);
    }
    
    // Convert to tile
    let tile = thought_to_tile(thought);
    info!("Bridging to {}: '{}'", tile.domain, tile.question.chars().take(60).collect::<String>());
    
    // Submit to PLATO
    match writer.submit_tile(&tile).await {
        Ok(response) => {
            info!("Submitted successfully: {} tiles now in {}", response.status, response.room);
            state.submitted_hashes.push(hash.to_string());
            state.last_sync = Some(Utc::now());
            Ok(true)
        }
        Err(e) => {
            warn!("Failed to submit: {}", e);
            Ok(false)
        }
    }
}

async fn run_bridge() -> Result<(), Box<dyn std::error::Error>> {
    let config = BridgeConfig::default();
    let writer = PlatoWriter::new(&config);
    let bidir = BidirSync::new(&config);
    
    // Load state
    let mut state = BidirSync::load_state(STATE_FILE);
    info!("Loaded bridge state with {} submitted hashes", state.submitted_hashes.len());
    
    // Get thoughts (try tensor.json, fall back to synthetic)
    let thoughts = match parse_tensor(&config.tensor_path) {
        Ok(t) => {
            info!("Loaded {} thoughts from {}", t.len(), config.tensor_path);
            t
        }
        Err(_) => {
            info!("No tensor.json found, using synthetic test data");
            synthetic_tensor()
        }
    };
    
    // Process thoughts
    let mut submitted = 0;
    let mut skipped = 0;
    let mut quality_filtered = 0;
    
    for (i, thought) in thoughts.iter().enumerate() {
        let hash = hash_thought(thought);
        
        // Check if already submitted
        if state.submitted_hashes.contains(&hash) {
            info!("Thought {} already submitted, skipping", i);
            skipped += 1;
            continue;
        }
        
        match submit_thought(&writer, thought, &hash, &mut state).await {
            Ok(true) => submitted += 1,
            Ok(false) => quality_filtered += 1,
            Err(e) => warn!("Error processing thought {}: {}", i, e),
        }
    }
    
    // Save state
    BidirSync::save_state(STATE_FILE, &state)?;
    info!("Bridge run complete: {} submitted, {} skipped, {} quality-filtered", submitted, skipped, quality_filtered);
    
    println!("\n=== Murmur-PLATO Bridge Summary ===");
    println!("Submitted: {}", submitted);
    println!("Skipped (duplicate): {}", skipped);
    println!("Filtered (quality gate): {}", quality_filtered);
    println!("================================\n");
    
    Ok(())
}

fn main() {
    init_logging();
    
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    if let Err(e) = rt.block_on(run_bridge()) {
        eprintln!("Bridge error: {}", e);
        process::exit(1);
    }
}
