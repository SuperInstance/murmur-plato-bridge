# murmur-plato-bridge


![CI](https://github.com/SuperInstance/murmur-plato-bridge/actions/workflows/rust-ci.yml/badge.svg)
Bridge between Murmur-agent's knowledge tensor and PLATO fleet rooms.

## What It Does

Murmur-agent generates thoughts that live in `tensor.json`. This bridge writes those thoughts to PLATO rooms as tiles, making them available to all fleet agents.

## Architecture

```
Murmur-agent → tensor.json → murmur-plato-bridge → PLATO rooms → Fleet agents
                                    ↑
                            Bidirectional:
Fleet tiles → murmur-plato-bridge → tensor.json (context for next session)
```

## Thought → Tile Mapping

| Murmur Strategy | PLATO Domain | Tag |
|-----------------|--------------|-----|
| explore | fleet_math_insights | exploration |
| contradict | fleet_math_tensions | contradiction |
| synthesize | fleet_math_synthesis | synthesis |
| question | fleet_math_open | open-question |
| connect | fleet_math_bridges | bridge |

## Content Parsing

The bridge extracts question/answer pairs from prose thought content:

- `"X implies Y"` → question: "What does X imply?", answer: Y
- `"X contradicts Y"` → question: "What tension exists?", answer: "The tension is..."
- `"What about X?"` → question: X, answer: "[open — investigation needed]"

## Building

```bash
cargo build --release
```

## Running

```bash
# Bridge thoughts from tensor.json to PLATO
cargo run --release

# With custom paths
PLATO_URL=http://localhost:8847 TENSOR_PATH=/path/to/tensor.json cargo run --release
```

## Testing

```bash
cargo test
```

## Configuration

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| PLATO_URL | http://localhost:8847 | PLATO API endpoint |
| TENSOR_PATH | tensor.json | Path to Murmur tensor.json |
| CONFIDENCE_THRESHOLD | 0.7 | Min confidence to bridge |
| MIN_CONTENT_LEN | 50 | Min content length |

## State

The bridge tracks submitted thought hashes in `bridge_state.json` to avoid re-submitting duplicates.
