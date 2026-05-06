use crate::{BridgeConfig, SubmitResponse, Tile, TileResponse, RoomInfo};
use reqwest::Client;
use serde_json;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlatoError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("JSON parse failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("PLATO rejected tile: {reason} (gate: {gate})")]
    Rejected { reason: String, gate: String },
    #[error("PLATO room not found: {0}")]
    RoomNotFound(String),
    #[error("Invalid response format")]
    InvalidResponse,
}

pub struct PlatoWriter {
    client: Client,
    base_url: String,
}

impl PlatoWriter {
    pub fn new(config: &BridgeConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        Self {
            client,
            base_url: config.plato_url.clone(),
        }
    }

    /// Submit a tile to PLATO
    pub async fn submit_tile(&self, tile: &Tile) -> Result<SubmitResponse, PlatoError> {
        let url = format!("{}/submit", self.base_url);
        let response = self.client
            .post(&url)
            .json(tile)
            .send()
            .await?;
        
        let text = response.text().await?;
        tracing::debug!("PLATO submit raw response ({}): {}", text.len(), &text[..text.len().min(200)]);
        
        let result: SubmitResponse = serde_json::from_str(&text)?;
        if result.status == "rejected" {
            return Err(PlatoError::Rejected {
                reason: result.reason.clone(),
                gate: result.gate.clone(),
            });
        }
        Ok(result)
    }

    /// Get tiles from a specific room
    pub async fn get_room_tiles(&self, room: &str) -> Result<RoomInfo, PlatoError> {
        let url = format!("{}/room/{}", self.base_url, room);
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if response.status() == 404 {
            return Err(PlatoError::RoomNotFound(room.to_string()));
        }
        
        let info: RoomInfo = response.json().await?;
        Ok(info)
    }

    /// Get tiles from a room since a timestamp
    pub async fn get_room_tiles_since(&self, room: &str, _since_timestamp: i64) -> Result<Vec<TileResponse>, PlatoError> {
        let info = self.get_room_tiles(room).await?;
        let filtered: Vec<TileResponse> = info.tiles.into_iter().filter(|_t| {
            // Filter by timestamp if provenance available
            true // simplified — real impl would check provenance.timestamp
        }).collect();
        Ok(filtered)
    }

    /// List all available rooms
    pub async fn list_rooms(&self) -> Result<Vec<String>, PlatoError> {
        let url = format!("{}/", self.base_url);
        let response = self.client.get(&url).send().await?;
        let _text = response.text().await?;
        // Parse room names from response or return empty
        Ok(vec![]) // Simplified — real impl would parse actual API
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plato_writer_creation() {
        let config = BridgeConfig::default();
        let writer = PlatoWriter::new(&config);
        assert_eq!(writer.base_url, "http://localhost:8847");
    }
}
