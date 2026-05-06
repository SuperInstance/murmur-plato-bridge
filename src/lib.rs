pub mod conflict_detector;
pub mod plato_writer;
pub mod quality_gate;
pub mod tensor_parser;
pub mod tile_mapper;
pub mod bidir_sync;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thought {
    pub content: String,
    #[serde(default)]
    pub connections: Vec<String>,
    #[serde(default)]
    pub questions: Vec<String>,
    pub confidence: f64,
    pub strategy: Strategy,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Strategy {
    Explore,
    Contradict,
    Synthesize,
    Question,
    Connect,
}

impl std::fmt::Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Strategy::Explore => write!(f, "explore"),
            Strategy::Contradict => write!(f, "contradict"),
            Strategy::Synthesize => write!(f, "synthesize"),
            Strategy::Question => write!(f, "question"),
            Strategy::Connect => write!(f, "connect"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub domain: String,
    pub question: String,
    pub answer: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RoomInfo {
    pub room: String,
    pub tile_count: usize,
    #[serde(default)]
    pub tiles: Vec<TileResponse>,
}

#[derive(Debug, Deserialize)]
pub struct TileResponse {
    pub question: String,
    pub answer: String,
    pub domain: String,
    #[serde(rename = "_hash")]
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct SubmitResponse {
    pub status: String,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub room: String,
    #[serde(default)]
    pub gate: String,
    #[serde(default)]
    pub tile_hash: Option<String>,
    #[serde(default)]
    pub room_tile_count: Option<u64>,
    #[serde(default)]
    pub provenance: Option<Provenance>,
    #[serde(default)]
    pub trace_id: Option<String>,
}


#[derive(Debug, Deserialize)]
pub struct Provenance {
    #[serde(default)]
    pub signed: Option<bool>,
    #[serde(default)]
    pub chain_size: Option<u64>,
    #[serde(default)]
    pub tile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TensorManifest {
    pub thoughts: Vec<Thought>,
    #[serde(default)]
    pub hashes: Vec<String>,
}

impl TensorManifest {
    pub fn from_path(path: &str) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

#[derive(Debug, Clone)]
pub struct BridgeConfig {
    pub plato_url: String,
    pub tensor_path: String,
    pub confidence_threshold: f64,
    pub min_content_len: usize,
    pub similarity_threshold: f64,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            plato_url: "http://localhost:8847".to_string(),
            tensor_path: "tensor.json".to_string(),
            confidence_threshold: 0.7,
            min_content_len: 50,
            similarity_threshold: 0.85,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BridgeState {
    pub submitted_hashes: Vec<String>,
    #[serde(default)]
    pub last_sync: Option<DateTime<Utc>>,
}

pub fn hash_thought(thought: &Thought) -> String {
    let mut hasher = Sha256::new();
    hasher.update(thought.content.as_bytes());
    hasher.update(thought.strategy.to_string().as_bytes());
    hex::encode(hasher.finalize())
}

pub use conflict_detector::hash_thought as compute_thought_hash;
pub use plato_writer::PlatoWriter;
pub use tensor_parser::new_thought_indices;
pub use tensor_parser::parse_tensor;
