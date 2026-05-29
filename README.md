# cache-layer-optimizer — Multi-Tier Cache Optimization

**L1/L2/L3 cache management with intelligent eviction, placement policies, and per-tier metrics.**

## What This Gives You

- **Multi-tier caching** — L1 (fast/small), L2 (medium), L3 (slow/large) with configurable sizes
- **Eviction policies** — LRU, LFU, FIFO, TTL-based, and adaptive eviction
- **Placement policies** — intelligent data placement across tiers based on access patterns
- **Hit-rate optimization** — automatic tier promotion/demotion based on access frequency
- **Per-tier metrics** — hit rate, miss rate, latency, and size tracking
- **Rust benches** — benchmarked for performance-critical paths

## Quick Start

```bash
pip install cache-layer-optimizer
```

```python
from cache_layer_optimizer import MultiTierCache, CacheLayer, EvictionPolicy, CacheOptimizer

# Set up a 3-tier cache
cache = MultiTierCache(layers=[
    CacheLayer(name="L1", max_size=100, eviction=EvictionPolicy.LRU),
    CacheLayer(name="L2", max_size=1000, eviction=EvictionPolicy.LFU),
    CacheLayer(name="L3", max_size=10000, eviction=EvictionPolicy.TTL),
])

# Use the cache
cache.put("model:response:hash123", {"text": "Hello", "cost": 0.001})
result = cache.get("model:response:hash123")

# Optimize placement
optimizer = CacheOptimizer(cache)
result = optimizer.optimize()
print(result.tier_adjustments)
print(result.projected_hit_rate)

# Check stats
for tier in cache.stats():
    print(f"{tier.name}: hit_rate={tier.hit_rate:.2%}, size={tier.current_size}")
```

## API Reference

### `MultiTierCache(layers)` — `put(key, value)`, `get(key)`, `invalidate(key)`, `stats()`
### `CacheLayer(name, max_size, eviction)` · `EvictionPolicy` — LRU, LFU, FIFO, TTL, ADAPTIVE
### `CacheOptimizer(cache)` — `optimize() → OptimizationResult`
### `PlacementPolicy` — `decide(key, access_pattern) → PlacementDecision`
### `TierStats` — hit_rate, miss_rate, latency_avg, current_size

## How It Fits

The caching layer for the [SuperInstance fleet](https://github.com/SuperInstance). Optimizes [api-gateway-1](https://github.com/SuperInstance/api-gateway-1) response caching and [Claude-PRISM-CF](https://github.com/SuperInstance/Claude-PRISM-CF) edge caching.

- **[api-gateway-1](https://github.com/SuperInstance/api-gateway-1)** — API gateway (uses cache for responses)
- **[Claude-PRISM-CF](https://github.com/SuperInstance/Claude-PRISM-CF)** — Edge caching
- **[cocapn-sdk](https://github.com/SuperInstance/cocapn-sdk)** — Model response caching

## Testing

```bash
pytest tests/
```

## Installation

```bash
pip install cache-layer-optimizer
```

Python 3.10+. MIT license.
