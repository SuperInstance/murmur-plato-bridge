use crate::{Thought, TensorManifest};
use std::collections::HashMap;
use std::io;

/// Parse tensor.json and extract Thought structs
pub fn parse_tensor(path: &str) -> io::Result<Vec<Thought>> {
    let manifest = TensorManifest::from_path(path)?;
    Ok(manifest.thoughts)
}

/// Extract connections graph (theorem-id → [connected-ids])
pub fn connections_graph(thoughts: &[Thought]) -> HashMap<String, Vec<String>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for (i, thought) in thoughts.iter().enumerate() {
        let id = format!("thought_{}", i);
        let mut connected_ids = Vec::new();
        for conn in &thought.connections {
            if !conn.starts_with("theorem_") && !conn.starts_with("thought_") {
                connected_ids.push(format!("theorem_{}", conn));
            } else {
                connected_ids.push(conn.clone());
            }
        }
        if !connected_ids.is_empty() {
            graph.insert(id, connected_ids);
        }
    }
    graph
}

/// Get indices of thoughts that are new (not in existing state hashes)
pub fn new_thought_indices(thoughts: &[Thought], existing_hashes: &[String]) -> Vec<usize> {
    thoughts
        .iter()
        .enumerate()
        .filter(|(_, t)| !existing_hashes.contains(&crate::hash_thought(t)))
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Strategy, Thought};
    use chrono::Utc;

    fn synthetic_tensor() -> Vec<Thought> {
        vec![
            Thought {
                content: "Tensor products provide a universal construction for bilinear maps across mathematical domains.".to_string(),
                connections: vec!["theorem_universal_property".to_string()],
                questions: vec![],
                confidence: 0.92,
                strategy: Strategy::Explore,
                timestamp: Utc::now(),
            },
            Thought {
                content: "The tension between classical set-theoretic foundations and category-theoretic abstraction reveals deep epistemological differences in mathematical practice.".to_string(),
                connections: vec!["theorem_cantor".to_string(), "theorem_klein_erlangen".to_string()],
                questions: vec!["How do we reconcile these foundational frameworks?".to_string()],
                confidence: 0.78,
                strategy: Strategy::Contradict,
                timestamp: Utc::now(),
            },
            Thought {
                content: "What if adjunction and coherence conditions are two perspectives on the same underlying mathematical reality?".to_string(),
                connections: vec![],
                questions: vec!["What deeper structure connects these concepts?".to_string()],
                confidence: 0.65,
                strategy: Strategy::Question,
                timestamp: Utc::now(),
            },
        ]
    }

    #[test]
    fn test_parse_synthetic_tensor() {
        let thoughts = synthetic_tensor();
        assert_eq!(thoughts.len(), 3);
        assert_eq!(thoughts[0].strategy, Strategy::Explore);
        assert_eq!(thoughts[1].strategy, Strategy::Contradict);
        assert_eq!(thoughts[2].strategy, Strategy::Question);
    }

    #[test]
    fn test_connections_graph() {
        let thoughts = synthetic_tensor();
        let graph = connections_graph(&thoughts);
        assert!(graph.contains_key("thought_0"));
        assert!(graph.contains_key("thought_1"));
        assert!(!graph.contains_key("thought_2")); // no connections
    }

    #[test]
    fn test_new_thought_indices() {
        let thoughts = synthetic_tensor();
        let hashes: Vec<String> = vec![crate::hash_thought(&thoughts[0])];
        let new = new_thought_indices(&thoughts, &hashes);
        assert_eq!(new.len(), 2); // thoughts 1 and 2 are new
        assert_eq!(new[0], 1);
        assert_eq!(new[1], 2);
    }
}
