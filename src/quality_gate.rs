use crate::{BridgeConfig, Thought};

/// Quality gate — determines if a thought should be bridged to PLATO
pub fn passes_gate(thought: &Thought, config: &BridgeConfig) -> bool {
    // Confidence threshold check
    if thought.confidence < config.confidence_threshold {
        tracing::debug!(
            "Thought rejected by confidence gate: {:.2} < {:.2}",
            thought.confidence,
            config.confidence_threshold
        );
        return false;
    }
    
    // Minimum content length check
    if thought.content.len() < config.min_content_len {
        tracing::debug!(
            "Thought rejected by length gate: {} < {}",
            thought.content.len(),
            config.min_content_len
        );
        return false;
    }
    
    // Content quality checks
    if !is_substantive(&thought.content) {
        tracing::debug!("Thought rejected: content not substantive");
        return false;
    }
    
    true
}

/// Check if content is substantive (not just placeholder/todo text)
fn is_substantive(content: &str) -> bool {
    let placeholder_patterns = [
        "todo",
        "tbd",
        "placeholder",
        "fixme",
        "xxx",
        "undefined",
        "not yet implemented",
    ];
    
    let content_lower = content.to_lowercase();
    
    for pattern in placeholder_patterns {
        if content_lower.contains(pattern) {
            return false;
        }
    }
    
    // Must have at least 2 words (after stripping very short content)
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.len() < 3 {
        return false;
    }
    
    true
}

/// Compute a quality score for a thought (for ranking/filtering)
pub fn quality_score(thought: &Thought, config: &BridgeConfig) -> f64 {
    let mut score = thought.confidence;
    
    // Boost for longer content (more informative)
    let content_boost = (thought.content.len() as f64 / 500.0).min(0.2);
    score += content_boost;
    
    // Boost for having questions (curiosity indicator)
    if !thought.questions.is_empty() {
        score += 0.1;
    }
    
    // Boost for having connections (integrated knowledge)
    if !thought.connections.is_empty() {
        score += 0.1;
    }
    
    score.min(1.0)
}

/// Filter thoughts by quality score
pub fn filter_by_quality<'a>(thoughts: &'a [Thought], config: &BridgeConfig) -> Vec<&'a Thought> {
    thoughts
        .iter()
        .filter(|t| passes_gate(t, config))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Strategy;
    use chrono::Utc;

    fn make_config() -> BridgeConfig {
        BridgeConfig {
            confidence_threshold: 0.7,
            min_content_len: 50,
            similarity_threshold: 0.85,
            ..Default::default()
        }
    }

    #[test]
    fn test_passes_gate_high_confidence() {
        let config = make_config();
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
        let config = make_config();
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
        let config = make_config();
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
    fn test_fails_gate_placeholder() {
        let config = make_config();
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
    fn test_quality_score() {
        let config = make_config();
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

    #[test]
    fn test_is_substantive() {
        assert!(is_substantive("This is a substantive thought about tensor products."));
        assert!(!is_substantive("TODO"));
        assert!(!is_substantive("Fixme"));
        assert!(!is_substantive("ab"));
    }
}
