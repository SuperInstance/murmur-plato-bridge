use crate::{Thought, Tile, Strategy};
use regex::Regex;
use once_cell::sync::Lazy;

/// Strategy → Domain mapping
pub fn strategy_to_domain(strategy: &Strategy) -> &'static str {
    match strategy {
        Strategy::Explore => "fleet_math_insights",
        Strategy::Contradict => "fleet_math_tensions",
        Strategy::Synthesize => "fleet_math_synthesis",
        Strategy::Question => "fleet_math_open",
        Strategy::Connect => "fleet_math_bridges",
    }
}

/// Strategy → Primary tag
pub fn strategy_to_tag(strategy: &Strategy) -> &'static str {
    match strategy {
        Strategy::Explore => "exploration",
        Strategy::Contradict => "contradiction",
        Strategy::Synthesize => "synthesis",
        Strategy::Question => "open-question",
        Strategy::Connect => "bridge",
    }
}

/// Patterns for content parsing
static IMPLIES_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(.+?)\s+implies\s+(.+)").unwrap());
static CONTRADICTS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(.+?)\s+contradicts?\s+(.+)").unwrap());
static DEPENDS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(.+?)\s+depends\s+on\s+(.+)").unwrap());
static ANALOGOUS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(.+?)\s+is\s+analogous\s+to\s+(.+)").unwrap());
static WHAT_IF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"What\s+(?:about|if)\s+(.+?)\??$").unwrap());
static TENSION_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"tension\s+between\s+(.+?)\s+and\s+(.+?)(?:\.|$)").unwrap());

/// Split thought content into question and answer based on patterns
pub fn parse_content(content: &str) -> (String, String) {
    let trimmed = content.trim();
    
    // "X implies Y" → question="What does X imply?", answer=Y
    if let Some(caps) = IMPLIES_RE.captures(trimmed) {
        let x = caps.get(1).map(|m| m.as_str()).unwrap_or("this");
        let y = caps.get(2).map(|m| m.as_str()).unwrap_or("something");
        return (format!("What does {} imply?", x), y.to_string());
    }
    
    // "X contradicts Y" → question="What tension exists between X and Y?"
    if let Some(caps) = CONTRADICTS_RE.captures(trimmed) {
        let x = caps.get(1).map(|m| m.as_str()).unwrap_or("this");
        let y = caps.get(2).map(|m| m.as_str()).unwrap_or("that");
        return (format!("What tension exists between {} and {}?", x, y), 
                format!("The tension is that {} contradicts {}.", x, y));
    }
    
    // "X depends on Y" → question="What does X depend on?"
    if let Some(caps) = DEPENDS_RE.captures(trimmed) {
        let x = caps.get(1).map(|m| m.as_str()).unwrap_or("this");
        let y = caps.get(2).map(|m| m.as_str()).unwrap_or("something");
        return (format!("What does {} depend on?", x), y.to_string());
    }
    
    // "X is analogous to Y" → question="What is X analogous to?"
    if let Some(caps) = ANALOGOUS_RE.captures(trimmed) {
        let x = caps.get(1).map(|m| m.as_str()).unwrap_or("this");
        let y = caps.get(2).map(|m| m.as_str()).unwrap_or("something");
        return (format!("What is {} analogous to?", x), y.to_string());
    }
    
    // "What about X?" or "What if X?" → question=X, answer="[open]"
    if let Some(caps) = WHAT_IF_RE.captures(trimmed) {
        let x = caps.get(1).map(|m| m.as_str()).unwrap_or("this");
        return (x.to_string(), "[open — investigation needed]".to_string());
    }
    
    // Default: use content as answer, generate question from strategy
    let question = format!("What insight emerges from this thought?");
    (question, trimmed.to_string())
}

/// Detect additional tags from content keywords
pub fn detect_keyword_tags(content: &str) -> Vec<String> {
    let content_lower = content.to_lowercase();
    let mut tags = Vec::new();
    
    let keywords = [
        ("tensor", "tensor"),
        ("category", "category-theory"),
        ("adjunction", "adjunction"),
        ("coherence", "coherence"),
        ("topology", "topology"),
        ("algebra", "algebra"),
        ("geometry", "geometry"),
        ("logic", "logic"),
        ("foundation", "foundations"),
        ("abstraction", "abstraction"),
        ("universal", "universal-property"),
        ("bilinear", "bilinear"),
    ];
    
    for (keyword, tag) in keywords {
        if content_lower.contains(keyword) {
            tags.push(tag.to_string());
        }
    }
    
    tags
}

/// Convert a Thought to a Tile
pub fn thought_to_tile(thought: &Thought) -> Tile {
    let domain = strategy_to_domain(&thought.strategy).to_string();
    let (question, answer) = parse_content(&thought.content);
    let strategy_tag = strategy_to_tag(&thought.strategy).to_string();
    let keyword_tags = detect_keyword_tags(&thought.content);
    
    let mut tags = vec![strategy_tag];
    tags.extend(keyword_tags);
    tags.push("murmur-bridge".to_string());
    
    // Normalize tags to lowercase-kebab-case
    tags = tags.into_iter().map(|t| {
        t.to_lowercase().split_whitespace().collect::<Vec<_>>().join("-")
    }).collect();
    
    Tile {
        domain,
        question,
        answer,
        tags,
    }
}

/// Convert a Tile back to a synthetic Thought (for bidir sync)
pub fn tile_to_thought(tile: &crate::TileResponse, strategy_hint: Strategy) -> Thought {
    let content = format!("{} Answer: {}", tile.question, tile.answer);
    Thought {
        content,
        connections: vec![],
        questions: vec![tile.question.clone()],
        confidence: 0.5, // default confidence for PLATO-sourced thoughts
        strategy: strategy_hint,
        timestamp: chrono::Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Strategy;
    use chrono::Utc;

    #[test]
    fn test_strategy_to_domain() {
        assert_eq!(strategy_to_domain(&Strategy::Explore), "fleet_math_insights");
        assert_eq!(strategy_to_domain(&Strategy::Contradict), "fleet_math_tensions");
        assert_eq!(strategy_to_domain(&Strategy::Synthesize), "fleet_math_synthesis");
        assert_eq!(strategy_to_domain(&Strategy::Question), "fleet_math_open");
        assert_eq!(strategy_to_domain(&Strategy::Connect), "fleet_math_bridges");
    }

    #[test]
    fn test_parse_content_implies() {
        let (q, a) = parse_content("Tensor products implies a universal property for bilinear maps");
        assert!(q.contains("What does"));
        assert!(a.contains("a universal property"));
    }

    #[test]
    fn test_parse_content_contradicts() {
        let (q, a) = parse_content("Classical set theory contradicts category theory in its treatment of infinite sets");
        assert!(q.contains("tension"));
        assert!(a.contains("contradicts"));
    }

    #[test]
    fn test_parse_content_what_about() {
        let (q, a) = parse_content("What about the role of adjunctions in coherence theorems?");
        assert_eq!(q, "the role of adjunctions in coherence theorems");
        assert_eq!(a, "[open — investigation needed]");
    }

    #[test]
    fn test_parse_content_default() {
        let (q, a) = parse_content("Tensor algebras and symmetric algebras are built from tensor products as foundational constructions.");
        assert_eq!(q, "What insight emerges from this thought?");
        assert!(a.contains("Tensor algebras"));
    }

    #[test]
    fn test_detect_keyword_tags() {
        let tags = detect_keyword_tags("Tensor products and category theory provide universal constructions");
        assert!(tags.contains(&"tensor".to_string()));
        assert!(tags.contains(&"category-theory".to_string()));
        assert!(tags.contains(&"universal-property".to_string()));
    }

    #[test]
    fn test_thought_to_tile() {
        let thought = Thought {
            content: "Tensor products implies a universal property for bilinear maps".to_string(),
            connections: vec!["theorem_universal".to_string()],
            questions: vec![],
            confidence: 0.9,
            strategy: Strategy::Explore,
            timestamp: Utc::now(),
        };
        let tile = thought_to_tile(&thought);
        assert_eq!(tile.domain, "fleet_math_insights");
        assert!(tile.tags.contains(&"exploration".to_string()));
        assert!(tile.tags.contains(&"murmur-bridge".to_string()));
    }
}
