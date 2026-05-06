use crate::{Thought, Tile, TileResponse};
use sha2::{Sha256, Digest};
use std::collections::HashSet;

/// Compute a hash for a thought (for deduplication)
pub fn hash_thought(thought: &Thought) -> String {
    let mut hasher = Sha256::new();
    hasher.update(thought.content.as_bytes());
    hasher.update(thought.strategy.to_string().as_bytes());
    hex::encode(hasher.finalize())
}

/// Compute a simple hash for tile content (for deduplication)
pub fn hash_tile(tile: &Tile) -> String {
    let mut hasher = Sha256::new();
    hasher.update(tile.question.as_bytes());
    hasher.update(tile.answer.as_bytes());
    hex::encode(hasher.finalize())
}

/// Compute similarity between two strings using Jaccard similarity on words
pub fn jaccard_similarity(a: &str, b: &str) -> f64 {
    let a_words: HashSet<&str> = a.split_whitespace().collect();
    let b_words: HashSet<&str> = b.split_whitespace().collect();
    
    let intersection = a_words.intersection(&b_words).count();
    let union = a_words.union(&b_words).count();
    
    if union == 0 {
        return 1.0;
    }
    intersection as f64 / union as f64
}

/// Check if a thought is likely a duplicate of existing tiles
pub fn is_duplicate(thought: &Thought, existing_tiles: &[TileResponse], threshold: f64) -> bool {
    let thought_hash = hash_thought(thought);
    
    // Hash-based check
    for tile in existing_tiles {
        let tile_hash = compute_tile_response_hash(tile);
        if thought_hash == tile_hash {
            return true; // exact hash match
        }
    }
    
    // Content similarity check (if hash didn't match)
    for tile in existing_tiles {
        let combined = format!("{} {}", tile.question, tile.answer);
        let similarity = jaccard_similarity(&thought.content, &combined);
        if similarity >= threshold {
            return true;
        }
    }
    
    false
}

/// Compute hash for a TileResponse (for dedup against stored tiles)
fn compute_tile_response_hash(tile: &TileResponse) -> String {
    let mut hasher = Sha256::new();
    hasher.update(tile.question.as_bytes());
    hasher.update(tile.answer.as_bytes());
    hex::encode(hasher.finalize())
}

/// Check if a new thought has novel content compared to existing thoughts
pub fn is_novel(thought: &Thought, all_thoughts: &[Thought]) -> bool {
    let thought_hash = hash_thought(thought);
    for existing in all_thoughts {
        if hash_thought(existing) == thought_hash {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Strategy, Thought};
    use chrono::Utc;

    #[test]
    fn test_jaccard_similarity() {
        let a = "tensor products provide universal construction";
        let b = "tensor products give universal property";
        let sim = jaccard_similarity(a, b);
        assert!(sim > 0.2, "Should have some overlap: {}", sim);
    }

    #[test]
    fn test_jaccard_identical() {
        let a = "tensor products are fundamental";
        let b = "tensor products are fundamental";
        assert_eq!(jaccard_similarity(a, b), 1.0);
    }

    #[test]
    fn test_jaccard_different() {
        let a = "tensor products";
        let b = "category theory adjunctions";
        let sim = jaccard_similarity(a, b);
        assert!(sim < 0.3, "Should have little overlap");
    }

    #[test]
    fn test_hash_thought() {
        let thought = Thought {
            content: "Test content".to_string(),
            connections: vec![],
            questions: vec![],
            confidence: 0.8,
            strategy: Strategy::Explore,
            timestamp: Utc::now(),
        };
        let hash1 = hash_thought(&thought);
        let hash2 = hash_thought(&thought);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 hex is 64 chars
    }

    #[test]
    fn test_is_novel() {
        let t1 = Thought {
            content: "Original thought".to_string(),
            connections: vec![],
            questions: vec![],
            confidence: 0.8,
            strategy: Strategy::Explore,
            timestamp: Utc::now(),
        };
        let t2 = Thought {
            content: "New thought".to_string(),
            connections: vec![],
            questions: vec![],
            confidence: 0.9,
            strategy: Strategy::Explore,
            timestamp: Utc::now(),
        };
        assert!(is_novel(&t2, &[t1.clone()]));
        // Cannot test duplicate-of-itself due to borrow checker, so test two identical t1s
        assert!(!is_novel(&t1, &[t1.clone()])); // t1 duplicate
    }
}
