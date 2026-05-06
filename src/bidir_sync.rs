use crate::{plato_writer::PlatoWriter, BridgeConfig, BridgeState, Strategy, Thought, TileResponse};
use chrono::Utc;
use std::collections::HashMap;

/// Bidirectional sync between PLATO and Murmur tensor
pub struct BidirSync {
    writer: PlatoWriter,
    config: BridgeConfig,
}

/// Result of syncing from PLATO to tensor
pub struct SyncResult {
    pub tiles_synced: usize,
    pub thoughts_added: Vec<Thought>,
}

impl BidirSync {
    pub fn new(config: &BridgeConfig) -> Self {
        Self {
            writer: PlatoWriter::new(config),
            config: config.clone(),
        }
    }

    /// Sync recent tiles from PLATO rooms into tensor format
    /// Returns synthetic thoughts that can be prepended to tensor.json
    pub async fn sync_from_plato(&self, days_back: i64) -> Result<SyncResult, crate::plato_writer::PlatoError> {
        let mut all_thoughts = Vec::new();
        let rooms = self.get_fleet_math_rooms();
        
        for room in rooms {
            match self.writer.get_room_tiles(&room).await {
                Ok(room_info) => {
                    for tile in room_info.tiles {
                        if self.is_recent_tile(&tile) {
                            if let Some(thought) = self.tile_to_thought(&tile) {
                                tracing::info!(
                                    "Synced tile from {}: {}",
                                    room,
                                    tile.question.chars().take(50).collect::<String>()
                                );
                                all_thoughts.push(thought);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch room {}: {}", room, e);
                }
            }
        }
        
        Ok(SyncResult {
            tiles_synced: all_thoughts.len(),
            thoughts_added: all_thoughts,
        })
    }

    /// Get list of fleet_math rooms to sync from
    fn get_fleet_math_rooms(&self) -> Vec<String> {
        vec![
            "fleet_math_insights".to_string(),
            "fleet_math_tensions".to_string(),
            "fleet_math_synthesis".to_string(),
            "fleet_math_open".to_string(),
            "fleet_math_bridges".to_string(),
        ]
    }

    /// Check if a tile is recent (within days_back)
    fn is_recent_tile(&self, tile: &TileResponse) -> bool {
        // Simplified — real impl would check tile.provenance.timestamp
        // For now, accept all tiles from fleet_math rooms
        true
    }

    /// Convert a PLATO tile to a synthetic Thought
    fn tile_to_thought(&self, tile: &TileResponse) -> Option<Thought> {
        let strategy = self.domain_to_strategy(&tile.domain)?;
        let content = format!(
            "[from PLATO: {}] {} -> {}",
            tile.domain,
            tile.question,
            tile.answer
        );
        Some(Thought {
            content,
            connections: vec![],
            questions: vec![tile.question.clone()],
            confidence: 0.5, // PLATO tiles get neutral confidence
            strategy,
            timestamp: Utc::now(),
        })
    }

    /// Infer strategy from domain
    fn domain_to_strategy(&self, domain: &str) -> Option<Strategy> {
        match domain {
            "fleet_math_insights" => Some(Strategy::Explore),
            "fleet_math_tensions" => Some(Strategy::Contradict),
            "fleet_math_synthesis" => Some(Strategy::Synthesize),
            "fleet_math_open" => Some(Strategy::Question),
            "fleet_math_bridges" => Some(Strategy::Connect),
            _ => Some(Strategy::Explore), // default
        }
    }

    /// Update bridge state with new submission
    pub fn record_submission(&self, state: &mut BridgeState, hash: String) {
        state.submitted_hashes.push(hash);
        state.last_sync = Some(Utc::now());
    }

    /// Load state from file
    pub fn load_state(path: &str) -> BridgeState {
        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => BridgeState::default(),
        }
    }

    /// Save state to file
    pub fn save_state(path: &str, state: &BridgeState) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(state)?;
        std::fs::write(path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_to_strategy() {
        let sync = BidirSync::new(&BridgeConfig::default());
        assert_eq!(sync.domain_to_strategy("fleet_math_insights"), Some(Strategy::Explore));
        assert_eq!(sync.domain_to_strategy("fleet_math_tensions"), Some(Strategy::Contradict));
        assert_eq!(sync.domain_to_strategy("fleet_math_synthesis"), Some(Strategy::Synthesize));
        assert_eq!(sync.domain_to_strategy("fleet_math_open"), Some(Strategy::Question));
        assert_eq!(sync.domain_to_strategy("fleet_math_bridges"), Some(Strategy::Connect));
    }

    #[test]
    fn test_get_fleet_math_rooms() {
        let sync = BidirSync::new(&BridgeConfig::default());
        let rooms = sync.get_fleet_math_rooms();
        assert!(rooms.contains(&"fleet_math_insights".to_string()));
        assert!(rooms.contains(&"fleet_math_tensions".to_string()));
        assert_eq!(rooms.len(), 5);
    }
}
