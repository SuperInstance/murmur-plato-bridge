# Murmur-PLATO Bridge Specification

## Overview

The `murmur-plato-bridge` connects Murmur-agent's knowledge tensor to PLATO rooms, enabling fleet-wide knowledge sharing.

## Direction

**Phase 1:** Unidirectional (tensor → PLATO) — safer, less risk of conflicts
**Phase 2:** Bidirectional (PLATO → tensor read on startup) — Murmur loads recent relevant tiles as context before thinking

## Data Structures

### Thought (from Murmur tensor.json)
```json
{
  "content": "string",
  "connections": ["theorem-id", ...],
  "questions": ["string", ...],
  "confidence": 0.0-1.0,
  "strategy": "explore|contradict|synthesize|question|connect",
  "timestamp": "ISO8601"
}
```

### Tile (PLATO format)
```json
{
  "domain": "string",
  "question": "string",
  "answer": "string",
  "tags": ["string", ...]
}
```

## Thought → Tile Mapping

### Strategy → Domain routing
| Strategy   | Domain                    | Tag           |
|------------|---------------------------|---------------|
| explore    | fleet_math_insights       | exploration   |
| contradict | fleet_math_tensions       | contradiction |
| synthesize | fleet_math_synthesis      | synthesis     |
| question   | fleet_math_open           | open-question |
| connect    | fleet_math_bridges        | bridge        |

### Content → Question/Answer parsing
Pattern-based extraction:

| Pattern                     | Question                                    | Answer              |
|-----------------------------|---------------------------------------------|---------------------|
| "X implies Y"               | "What does X imply?"                        | Y                   |
| "X contradicts Y"           | "What tension exists between X and Y?"      | "The tension is..." |
| "What about X?"             | X                                           | "[open — investigation needed]" |
| "X depends on Y"            | "What does X depend on?"                     | Y                   |
| "X is analogous to Y"       | "What is X analogous to?"                   | Y                   |
| Default                     | "What insight emerges from this thought?"    | content (full prose)|

### Tag inheritance
- Tags propagate from: strategy, detected keywords, connected theorem IDs
- Normalized to lowercase-kebab-case

## Quality Gate

```rust
fn passes_gate(thought: &Thought) -> bool {
    thought.confidence >= 0.7          // confidence threshold
    && thought.content.len() >= 50    // non-trivial content
    && !is_duplicate(thought)          // novelty check
}
```

## Conflict Detection

1. **Semantic dedup:** Compare new thought content against existing tiles in target room
2. **Jaccard similarity:** If similarity > 0.85, skip (already tiled)
3. **Hash dedup:** If content hash matches existing tile, skip
4. **Merge strategy (future):** Same insight → update existing tile's reinforcement_count

## Bidirectional Sync (Phase 2)

### PLATO → Tensor (read on startup)
```
1. Murmur-agent starts session
2. Bridge queries PLATO rooms for domain ∈ {fleet_math_*}
3. Recent tiles (last 7 days) are converted to synthetic Thoughts
4. Synthetic Thoughts are prepended to tensor.json as "context tiles"
5. Murmur thinks with fleet awareness
```

### Thought → Tile (write on emit)
```
1. Murmur emits new Thought to tensor.json
2. Bridge detects new thought (hash tracking)
3. Thought passes quality gate?
4. Thought → Tile mapping
5. POST to PLATO /submit
6. Record hash to prevent re-submission
```

## Room Naming

Auto-detect target room from domain prefix:
- `fleet_math_*` → room = domain (e.g., `fleet_math_insights`)
- `fleet_code_*` → room = domain
- Default → `fleet_knowledge` (catch-all)

## API Endpoints

### PLATO Submit
```
POST localhost:8847/submit
Content-Type: application/json
{
  "domain": "fleet_math_insights",
  "question": "...",
  "answer": "...",
  "tags": ["exploration", "tensor-bridge"]
}
```

### PLATO Query (for bidir read)
```
GET localhost:8847/room/<room_name>/tiles?since=<timestamp>
```

## File Layout

```
murmur-plato-bridge/
├── Cargo.toml
├── src/
│   ├── lib.rs          # public API, module re-exports
│   ├── main.rs         # CLI entry point
│   ├── tensor_parser.rs   # tensor.json → Vec<Thought>
│   ├── tile_mapper.rs     # Thought → Tile
│   ├── plato_writer.rs    # POST to PLATO
│   ├── conflict_detector.rs # dedup, similarity check
│   ├── bidir_sync.rs      # bidirectional logic
│   └── quality_gate.rs    # confidence + novelty
├── tests/
│   ├── test_mapping.rs   # Thought → Tile mapping tests
│   └── test_quality.rs   # quality gate tests
├── SPEC.md
└── README.md
```

## Acceptance Criteria

1. Parses a real or synthetic `tensor.json` into `Thought` structs
2. Converts thoughts to tiles with correct domain/tag routing
3. Submits tiles to PLATO and receives acceptance (P0 gate)
4. Skips duplicate insights (conflict detection)
5. Respects confidence threshold (quality gate)
6. Logs all bridging activity for debugging
7. Bidirectional mode: on restart, loads recent fleet tiles as context

## Testing

### Unit Tests
- `test_mapping.rs`: Strategy → domain mapping correctness
- `test_quality.rs`: Confidence threshold, duplicate detection

### Integration Test
- Submit 3 synthetic thoughts to PLATO
- Verify tiles appear in correct rooms
- Verify hash-tracked deduplication works
