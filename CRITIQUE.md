# CRITIQUE.md — Murmur-PLATO Bridge

## What Was Built

A Rust crate that bridges Murmur-agent's knowledge tensor (`tensor.json`) to PLATO fleet rooms as tiles, with bidirectional read capability.

## Critical Evaluation

### Is the Thought → Tile mapping lossy?

**Yes, significantly.** Here's what's lost:

1. **Connections graph**: The `connections[]` array in Thought references other theorems/ideas by ID. This relationship data doesn't survive the translation — a Tile is standalone.

2. **Question multiplicity**: A Thought can have multiple questions. The mapping picks one pattern and generates a single question. The others are discarded.

3. **Strategy nuance**: `Strategy` is an enum but real thoughts often blend strategies (e.g., "explore" that also "contradicts" a prior belief).

4. **Temporal context**: Thoughts have timestamps and potentially causal ordering. Tiles are just stored, their temporal relationship to other thoughts is lost.

5. **Confidence distribution**: Confidence is a single float, but could represent uncertainty about different aspects of the thought.

**Mitigation potential**: Include `connections` as tagged references in the Tile's `tags` field (e.g., `theorem:theorem_universal_property`). This would preserve the graph structure in PLATO's tag space.

### Does the domain/tag routing make sense?

**The routing is functional but arbitrary.** `fleet_math_*` rooms are reasonable for mathematical thoughts, but:

- "insights/tensions/synthesis/bridges/open" are soft categories — the same thought could plausibly fit multiple.
- The routing is purely strategy-based, not content-based. A "contradict" thought about cooking methods routes to `fleet_math_tensions`, which is wrong.

**Better approach**: Content-based + strategy-based hybrid. Parse keywords from content to determine domain, use strategy only for the primary tag.

### What would make this bridge exceptional vs just "it works"?

1. **Thought chaining**: When a thought references another by ID, the bridge should query PLATO for that referenced tile and create a link. PLATO would gain an explicit knowledge graph.

2. **Bidirectional reinforcement**: When multiple agents read the same tile from PLATO and it influences their thinking, that influence should flow back to Murmur's tensor. Currently it's write-only from Murmur's perspective.

3. **Temporal batching**: Murmur generates thoughts in bursts. The bridge should batch these and submit together, with ordering preserved via `provenance.timestamp`.

4. **Confidence-weighted routing**: High-confidence thoughts → public fleet rooms. Low-confidence → a "draft" room that agents can review before publicizing.

5. **Semantic dedup at import**: Before bridging from PLATO to tensor, check if the tile's content would create a truly novel thought or just echo existing knowledge.

### Most valuable thing PLATO gains from Murmur

**Systematic, confidence-rated insights** — Murmur generates thoughts with explicit confidence scores and strategy tags. This gives PLATO a curated pipeline instead of raw user submissions.

### Most valuable thing Murmur gains from PLATO

**Fleet-wide context awareness** — Before thinking about tensor products, Murmur can query what other agents have already learned. This prevents redundant exploration and enables building on others' work.

### Bidirectional or unidirectional: which is more valuable?

**Bidirectional**, but with caveats.

- **Unidirectional (tensor → PLATO)** is safe and useful for broadcasting Murmur's insights.
- **Bidirectional (PLATO → tensor)** is more transformative — it makes Murmur aware of the fleet's knowledge.

However, the value depends on **semantic filtering**:
- If every PLATO tile gets converted to a synthetic thought, Murmur's tensor drowns in noise.
- The bridge needs to filter PLATO tiles by relevance (domain match) and novelty (doesn't duplicate existing tensor content).

## What's Working

- Thought parsing (tensor.json → Vec<Thought>)
- Strategy → domain routing
- Content pattern extraction (implies/contradicts/what about → question/answer)
- Quality gate (confidence threshold, content length)
- Hash-based deduplication
- Async PLATO submission with proper error handling
- 23 unit tests passing

## What Needs Work

1. **Tile → Thought lossy translation** — connections, multiple questions lost
2. **Content-based domain routing** — not just strategy-based
3. **PLATO response parsing** — needed debug output to diagnose duplicate detection
4. **Bidirectional sync** — implemented but not integrated into main loop
5. **State persistence** — bridge_state.json saves hashes but doesn't prevent re-runs well

## Verdict

The bridge is a solid foundation. The core mechanics (parsing, quality gate, submission) work. The mapping is intentionally lossy to match PLATO's tile model. For a v0.1, this is appropriate — the lossiness is documented and could be mitigated with tagging enhancements.

The most valuable improvement would be **proper bidirectional sync with semantic filtering**, which would make Murmur genuinely aware of fleet knowledge rather than just broadcasting into it.
