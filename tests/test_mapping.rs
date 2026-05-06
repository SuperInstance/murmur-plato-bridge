use murmur_plato_bridge::tile_mapper::{parse_content, strategy_to_domain, thought_to_tile};
use murmur_plato_bridge::{Thought, Strategy};
use chrono::Utc;

#[test]
fn test_implies_pattern() {
    let (q, a) = parse_content("Tensor products implies a universal property");
    assert!(q.contains("What does"));
    assert!(a.contains("universal property"));
}

#[test]
fn test_contradicts_pattern() {
    let (q, a) = parse_content("Set theory contradicts category theory in foundations");
    assert!(q.contains("tension"));
    assert!(a.contains("contradicts"));
}

#[test]
fn test_what_about_pattern() {
    let (q, a) = parse_content("What about adjunctions in coherence theorems?");
    assert_eq!(q, "adjunctions in coherence theorems");
    assert_eq!(a, "[open — investigation needed]");
}

#[test]
fn test_strategy_to_domain_explore() {
    assert_eq!(strategy_to_domain(&Strategy::Explore), "fleet_math_insights");
}

#[test]
fn test_strategy_to_domain_contradict() {
    assert_eq!(strategy_to_domain(&Strategy::Contradict), "fleet_math_tensions");
}

#[test]
fn test_thought_to_tile_with_questions() {
    let thought = Thought {
        content: "Tensor products implies a universal property for bilinear maps".to_string(),
        connections: vec!["theorem_universal".to_string()],
        questions: vec!["What is the universal property?".to_string()],
        confidence: 0.9,
        strategy: Strategy::Explore,
        timestamp: Utc::now(),
    };
    let tile = thought_to_tile(&thought);
    assert_eq!(tile.domain, "fleet_math_insights");
    assert!(tile.tags.contains(&"exploration".to_string()));
    assert!(tile.tags.contains(&"murmur-bridge".to_string()));
}
