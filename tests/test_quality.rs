use murmur_plato_bridge::quality_gate::{passes_gate, quality_score};
use murmur_plato_bridge::{BridgeConfig, Thought, Strategy};
use chrono::Utc;

fn default_config() -> BridgeConfig {
    BridgeConfig {
        confidence_threshold: 0.7,
        min_content_len: 50,
        similarity_threshold: 0.85,
        ..Default::default()
    }
}

#[test]
fn test_passes_gate_high_confidence() {
    let config = default_config();
    let thought = Thought {
        content: "Tensor products provide a universal construction for bilinear maps across mathematical domains.".to_string(),
        connections: vec!["theorem_universal".to_string()],
        questions: vec![],
        confidence: 0.92,
        strategy: Strategy::Explore,
        timestamp: Utc::now(),
    };
    assert!(passes_gate(&thought, &config));
}

#[test]
fn test_fails_gate_low_confidence() {
    let config = default_config();
    let thought = Thought {
        content: "This is a test thought with sufficient length to pass the content gate.".to_string(),
        connections: vec![],
        questions: vec![],
        confidence: 0.3,
        strategy: Strategy::Explore,
        timestamp: Utc::now(),
    };
    assert!(!passes_gate(&thought, &config));
}

#[test]
fn test_fails_gate_short_content() {
    let config = default_config();
    let thought = Thought {
        content: "Too short".to_string(),
        connections: vec![],
        questions: vec![],
        confidence: 0.95,
        strategy: Strategy::Explore,
        timestamp: Utc::now(),
    };
    assert!(!passes_gate(&thought, &config));
}

#[test]
fn test_fails_gate_placeholder_content() {
    let config = default_config();
    let thought = Thought {
        content: "This is TODO and needs to be fixed".to_string(),
        connections: vec![],
        questions: vec![],
        confidence: 0.95,
        strategy: Strategy::Explore,
        timestamp: Utc::now(),
    };
    assert!(!passes_gate(&thought, &config));
}

#[test]
fn test_quality_score_with_boosts() {
    let config = default_config();
    let thought = Thought {
        content: "Tensor products provide a universal construction for bilinear maps across mathematical domains.".to_string(),
        connections: vec!["theorem_universal".to_string()],
        questions: vec!["What about coherence?".to_string()],
        confidence: 0.8,
        strategy: Strategy::Explore,
        timestamp: Utc::now(),
    };
    let score = quality_score(&thought, &config);
    assert!(score > 0.8);
    assert!(score <= 1.0);
}
